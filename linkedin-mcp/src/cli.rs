use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "linkedin-mcp", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run OAuth flow and store tokens in the OS keychain.
    Auth(AuthArgs),
    /// Serve MCP over stdio.
    Serve(ServeArgs),
}

#[derive(clap::Args, Debug, Default)]
pub struct AuthArgs {
    #[arg(long, env = "LINKEDIN_CLIENT_ID")]
    pub client_id: Option<String>,
    #[arg(long, env = "LINKEDIN_CLIENT_SECRET")]
    pub client_secret: Option<String>,
    #[arg(long, default_value = "default")]
    pub account: String,
    #[arg(long, default_value = "keychain", value_parser = ["keychain", "file"])]
    pub token_store: String,
}

#[derive(clap::Args, Debug, Default)]
pub struct ServeArgs {
    #[arg(long, default_value = "default")]
    pub account: String,
    #[arg(long, default_value = "keychain", value_parser = ["keychain", "file"])]
    pub token_store: String,
}
