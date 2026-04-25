use std::process::Stdio;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::Command};

#[tokio::test]
async fn ping_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_linkedin-mcp"))
        .args(["serve", "--token-store", "file"])
        .env("HOME", tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn().unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();

    let init = serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0"}}
    });
    stdin.write_all(format!("{init}\n").as_bytes()).await.unwrap();
    let _init_reply = stdout.next_line().await.unwrap().unwrap();

    // MCP requires an "initialized" notification before tool calls.
    let initialized = serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"});
    stdin.write_all(format!("{initialized}\n").as_bytes()).await.unwrap();

    let call = serde_json::json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"ping","arguments":{}}
    });
    stdin.write_all(format!("{call}\n").as_bytes()).await.unwrap();
    let reply = stdout.next_line().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&reply).expect("invalid JSON reply");
    let text = v["result"]["content"][0]["text"].as_str().expect("no text content");
    let inner: serde_json::Value = serde_json::from_str(text).expect("content not JSON");
    assert_eq!(inner["pong"], true, "unexpected reply: {reply}");

    child.kill().await.unwrap();
}
