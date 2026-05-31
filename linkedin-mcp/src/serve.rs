use crate::cli::ServeArgs;
use crate::mcp::{server, tools::ToolRegistry, ping::Ping};
use crate::auth::storage::build_store;
use crate::linkedin::LinkedInClient;
use crate::tools::{
    auth_status::AuthStatus,
    comment_create::CommentCreate,
    comment_delete::CommentDelete,
    comment_list::CommentList,
    post_article::PostArticle,
    post_delete::PostDelete,
    post_document::PostDocument,
    post_get::PostGet,
    post_image::PostImage,
    post_multi_image::PostMultiImage,
    post_poll::PostPoll,
    post_reshare::PostReshare,
    post_text::PostText,
    post_update::PostUpdate,
    post_video::PostVideo,
    posts_list::PostsList,
    reaction_add::ReactionAdd,
    reaction_remove::ReactionRemove,
    whoami::WhoAmI,
};
use std::sync::Arc;

pub async fn run(args: ServeArgs) -> anyhow::Result<()> {
    let store: Arc<dyn crate::auth::TokenStore> = match &args.broker_url {
        Some(url) => {
            let token = args.broker_token.clone().ok_or_else(|| {
                anyhow::anyhow!("--broker-url requires --broker-token (or LINKEDIN_BROKER_TOKEN)")
            })?;
            tracing::info!(broker = %url, "using central auth broker");
            Arc::new(crate::auth::storage::RemoteStore::new(url.clone(), token))
        }
        None => build_store(&args.token_store)?.into(),
    };

    let mut registry = ToolRegistry::new();
    registry.register(Ping);

    // A broker fetch can fail transiently at startup; treat that like "no token"
    // (expose ping + auth-status) rather than crashing the server.
    let loaded = match store.load(&args.account).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(account = %args.account, error = %e, "failed to load token at startup");
            None
        }
    };

    match loaded {
        Some(record) => {
            // Warn early (structured, for alerting) if a human re-auth is due
            // soon, rather than letting a long-running agent fail mid-task.
            if record.needs_reauth_soon() {
                tracing::warn!(
                    account = %args.account,
                    refresh_expires_in_seconds = record.refresh_expires_in_seconds(),
                    "LinkedIn refresh token is missing or expiring soon — run `linkedin-mcp auth` to re-authenticate"
                );
            }
            if !store.is_remote() && args.client_secret.is_none() && record.refresh_token.is_some() {
                tracing::warn!(
                    "no LINKEDIN_CLIENT_SECRET set; access-token refresh will fail for confidential apps once the current token expires"
                );
            }
            let http = reqwest::Client::builder()
                .user_agent(concat!("linkedin-mcp/", env!("CARGO_PKG_VERSION")))
                .build()?;
            let client = Arc::new(LinkedInClient::new(
                http,
                record,
                store.clone(),
                args.account.clone(),
                args.client_secret.clone(),
            ));
            registry.register(AuthStatus { client: Some(client.clone()) });
            registry.register(WhoAmI { client: client.clone() });
            registry.register(PostText { client: client.clone() });
            registry.register(PostDelete { client: client.clone() });
            registry.register(PostGet { client: client.clone() });
            registry.register(PostUpdate { client: client.clone() });
            registry.register(PostArticle { client: client.clone() });
            registry.register(PostImage { client: client.clone() });
            registry.register(PostVideo { client: client.clone() });
            registry.register(PostsList { client: client.clone() });
            registry.register(CommentList { client: client.clone() });
            registry.register(CommentCreate { client: client.clone() });
            registry.register(CommentDelete { client: client.clone() });
            registry.register(ReactionAdd { client: client.clone() });
            registry.register(ReactionRemove { client: client.clone() });
            registry.register(PostPoll { client: client.clone() });
            registry.register(PostDocument { client: client.clone() });
            registry.register(PostMultiImage { client: client.clone() });
            registry.register(PostReshare { client: client.clone() });
        }
        None => {
            tracing::warn!(
                account = %args.account,
                "no token found; only the ping and linkedin-auth-status tools are available. Run `linkedin-mcp auth` first."
            );
            // Still expose status so an agent can discover it must authenticate.
            registry.register(AuthStatus { client: None });
        }
    }

    server::run(registry).await
}
