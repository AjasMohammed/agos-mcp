use clap::Parser;
use gmail_mcp_core::mcp::{McpServer, ServerInfo, StdioTransport, ToolRegistry};
use std::io;
use std::sync::Arc;

mod cli;
use cli::{AccountsCmd, Cli};
use gmail_mcp_core::auth::store::TokenStore;

async fn get_token_store(file_store: bool) -> Arc<dyn TokenStore> {
    if file_store || std::env::var("GMAIL_MCP_FILE_STORE").is_ok() {
        eprintln!("WARNING: Using encrypted file store fallback.");
        let home = dirs::home_dir().expect("no home dir");
        let path = home.join(".gmail-mcp-tokens.enc");
        let passphrase = std::env::var("GMAIL_MCP_PASSPHRASE")
            .unwrap_or_else(|_| "default-passphrase".to_string());
        Arc::new(
            gmail_mcp_core::auth::store::encrypted_file::EncryptedFileStore::new(path, passphrase),
        ) as Arc<dyn TokenStore>
    } else {
        Arc::new(gmail_mcp_core::auth::store::keychain::KeychainStore::new(
            "gmail-mcp",
        )) as Arc<dyn TokenStore>
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(io::stderr) // never stdout — that's MCP protocol territory
        .json()
        .init();

    match cli {
        Cli::Serve { account } => run_server(account).await,
        Cli::Auth {
            scopes,
            device,
            service_account,
            account,
            client_id,
            client_secret,
            file_store,
        } => {
            eprintln!("Auth flow started for account '{account}'");
            let client_id = client_id.unwrap_or_else(|| "default-client-id".to_string());
            let scopes_vec: Vec<String> = scopes.split(',').flat_map(|s| {
                match s.trim() {
                    "read" => vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
                    "write" => vec!["https://www.googleapis.com/auth/gmail.modify".to_string()],
                    "full" => vec!["https://mail.google.com/".to_string()],
                    other => vec![other.to_string()],
                }
            }).collect();

            if device || service_account.is_some() {
                eprintln!("Device and service_account flows are not fully implemented yet.");
                return Ok(());
            }

            let flow = gmail_mcp_core::auth::oauth::loopback::LoopbackFlow::new(
                client_id, client_secret, scopes_vec,
            );
            let tokens = flow.run().await.expect("OAuth loopback flow failed");

            let store = get_token_store(file_store).await;
            store
                .put(&account, &tokens)
                .await
                .expect("Failed to store tokens");
            eprintln!("Successfully authenticated and stored tokens for account '{account}'.");
            Ok(())
        }
        Cli::Accounts { cmd } => {
            match cmd {
                AccountsCmd::List => {
                    let store = get_token_store(false).await;
                    let accounts = store.list_accounts().await.unwrap_or_default();
                    if accounts.is_empty() {
                        println!(
                            "No accounts found (or listing is unsupported by keyring backend)."
                        );
                    } else {
                        for account in accounts {
                            println!("- {account}");
                        }
                    }
                }
            }
            Ok(())
        }
        Cli::Logout { account } => {
            let store = get_token_store(false).await;
            if let Err(e) = store.delete(&account).await {
                eprintln!("Failed to logout account '{account}': {e}");
            } else {
                println!("Logged out of account '{account}'");
            }
            Ok(())
        }
    }
}

async fn run_server(account: String) -> anyhow::Result<()> {
    let mut registry = ToolRegistry::new();

    let store = get_token_store(false).await;
    let client_id = std::env::var("GMAIL_MCP_CLIENT_ID")
        .unwrap_or_else(|_| "default-client-id".to_string());
    let tokens = std::sync::Arc::new(gmail_mcp_core::auth::TokenManager::new(
        store,
        client_id,
        account.clone(),
    ));
    let gmail = std::sync::Arc::new(gmail_mcp_core::gmail::Client::new(tokens).await);

    registry.register(Arc::new(gmail_mcp_core::tools::GmailSearchTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailReadTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailGetThreadTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailListLabelsTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailListFiltersTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailGetFilterTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailListDraftsTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailGetProfileTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(
        gmail_mcp_core::tools::GmailDownloadAttachmentTool {
            client: gmail.clone(),
        },
    ));

    // Phase 5 tools
    registry.register(Arc::new(gmail_mcp_core::tools::GmailSendTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailCreateDraftTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailUpdateDraftTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailSendDraftTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailDeleteDraftTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailModifyLabelsTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailTrashTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailUntrashTool {
        client: gmail.clone(),
    }));

    // Phase 6 tools
    registry.register(Arc::new(gmail_mcp_core::tools::GmailGetLabelTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailCreateLabelTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailUpdateLabelTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailDeleteLabelTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailGetOrCreateLabelTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailCreateFilterTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailDeleteFilterTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(
        gmail_mcp_core::tools::GmailCreateFilterFromTemplateTool {
            client: gmail.clone(),
        },
    ));
    registry.register(Arc::new(
        gmail_mcp_core::tools::GmailBatchModifyLabelsTool {
            client: gmail.clone(),
        },
    ));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailBatchTrashTool {
        client: gmail.clone(),
    }));
    registry.register(Arc::new(gmail_mcp_core::tools::GmailBatchDeleteTool {
        client: gmail.clone(),
    }));

    let registry = Arc::new(registry);

    let audit_sink = Arc::new(gmail_mcp_core::audit::AuditSink::new(Arc::new(
        gmail_mcp_core::audit::StderrJsonEmitter,
    )));

    let server = McpServer::new(
        registry,
        ServerInfo {
            name: "gmail-mcp",
            version: env!("CARGO_PKG_VERSION"),
        },
        account,
        Some(audit_sink),
    );

    let shutdown = server.shutdown_token();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown.cancel();
    });

    server.run(StdioTransport::new()).await
}
