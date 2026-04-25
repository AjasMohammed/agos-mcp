use crate::cli::ServeArgs;
use crate::mcp::{server, tools::ToolRegistry, ping::Ping};
use crate::auth::storage::build_store;
use crate::linkedin::LinkedInClient;
use crate::tools::{
    comment_create::CommentCreate,
    comment_delete::CommentDelete,
    comment_list::CommentList,
    post_article::PostArticle,
    post_delete::PostDelete,
    post_get::PostGet,
    post_image::PostImage,
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
    let store: Arc<dyn crate::auth::TokenStore> = build_store(&args.token_store)?.into();

    let mut registry = ToolRegistry::new();
    registry.register(Ping);

    match store.load(&args.account)? {
        Some(record) => {
            let http = reqwest::Client::builder()
                .user_agent(concat!("linkedin-mcp/", env!("CARGO_PKG_VERSION")))
                .build()?;
            let client = Arc::new(LinkedInClient::new(http, record, store.clone(), args.account.clone()));
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
        }
        None => {
            tracing::warn!(
                account = %args.account,
                "no token found; only the ping tool is available. Run `linkedin-mcp auth` first."
            );
        }
    }

    server::run(registry).await
}
