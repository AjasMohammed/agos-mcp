use assert_cmd::Command;

#[test]
fn prints_help() {
    Command::cargo_bin("linkedin-mcp")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("linkedin-mcp"));
}
