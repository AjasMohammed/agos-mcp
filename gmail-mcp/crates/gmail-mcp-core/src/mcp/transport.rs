use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct StdioTransport {
    stdin: BufReader<tokio::io::Stdin>,
    stdout: tokio::io::Stdout,
    line_buf: String,
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(tokio::io::stdin()),
            stdout: tokio::io::stdout(),
            line_buf: String::new(),
        }
    }

    /// Read one newline-delimited JSON-RPC message. `Ok(None)` on EOF.
    pub async fn recv(&mut self) -> std::io::Result<Option<JsonRpcRequest>> {
        self.line_buf.clear();
        let n = self.stdin.read_line(&mut self.line_buf).await?;
        if n == 0 {
            return Ok(None);
        }
        let req: JsonRpcRequest = serde_json::from_str(self.line_buf.trim())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(req))
    }

    pub async fn send(&mut self, resp: &JsonRpcResponse) -> std::io::Result<()> {
        let line = serde_json::to_string(resp)?;
        self.stdout.write_all(line.as_bytes()).await?;
        self.stdout.write_all(b"\n").await?;
        self.stdout.flush().await
    }
}
