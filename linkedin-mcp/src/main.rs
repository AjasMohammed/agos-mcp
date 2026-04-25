use anyhow::Result;
use clap::Parser;
use linkedin_mcp::cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .json()
        .init();

    match cli.command {
        Command::Auth(args) => linkedin_mcp::auth::run(args).await,
        Command::Serve(args) => linkedin_mcp::serve::run(args).await,
    }
}
