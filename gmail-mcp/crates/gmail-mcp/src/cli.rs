use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "gmail-mcp",
    version,
    about = "Standalone Production-Grade Gmail MCP Server"
)]
pub enum Cli {
    /// Start the MCP server over stdio
    Serve {
        /// Account name for multi-account setups.
        #[arg(long, default_value = "default")]
        account: String,
    },
    /// Authenticate a new Gmail account
    Auth {
        /// Scope preset: read, write, full.
        #[arg(long, default_value = "read")]
        scopes: String,

        /// Use device code flow (headless).
        #[arg(long)]
        device: bool,

        /// Use service account JSON file.
        #[arg(long, value_name = "PATH")]
        service_account: Option<std::path::PathBuf>,

        /// Account name for multi-account setups.
        #[arg(long, default_value = "default")]
        account: String,

        /// Override the OAuth client ID
        #[arg(long, env = "GMAIL_MCP_CLIENT_ID")]
        client_id: Option<String>,

        /// Override the OAuth client secret
        #[arg(long, env = "GMAIL_MCP_CLIENT_SECRET")]
        client_secret: Option<String>,

        /// Use encrypted file store instead of OS keychain.
        #[arg(long)]
        file_store: bool,
    },
    /// Manage accounts
    Accounts {
        #[command(subcommand)]
        cmd: AccountsCmd,
    },
    /// Logout from an account
    Logout {
        /// Account name for multi-account setups.
        #[arg(long, default_value = "default")]
        account: String,
    },
}

#[derive(Parser, Debug)]
pub enum AccountsCmd {
    /// List authenticated accounts
    List,
}
