use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TextPostBody<'a> {
    pub author: &'a str,
    pub commentary: &'a str,
    pub visibility: &'a str,        // "PUBLIC" | "CONNECTIONS" | "LOGGED_IN"
    pub distribution: Distribution,
    #[serde(rename = "lifecycleState")]
    pub lifecycle_state: &'a str,   // "PUBLISHED"
    #[serde(rename = "isReshareDisabledByAuthor")]
    pub is_reshare_disabled_by_author: bool,
}

#[derive(Debug, Serialize)]
pub struct MediaPostBody<'a> {
    pub author: &'a str,
    pub commentary: &'a str,
    pub visibility: &'a str,
    pub distribution: Distribution,
    #[serde(rename = "lifecycleState")]
    pub lifecycle_state: &'a str,
    #[serde(rename = "isReshareDisabledByAuthor")]
    pub is_reshare_disabled_by_author: bool,
    pub content: PostMediaContent<'a>,
}

#[derive(Debug, Serialize)]
pub struct PostMediaContent<'a> {
    pub media: PostMediaItem<'a>,
}

#[derive(Debug, Serialize)]
pub struct PostMediaItem<'a> {
    pub id: &'a str,
}

#[derive(Debug, Serialize)]
pub struct ArticlePostBody<'a> {
    pub author: &'a str,
    pub commentary: &'a str,
    pub visibility: &'a str,
    pub distribution: Distribution,
    #[serde(rename = "lifecycleState")]
    pub lifecycle_state: &'a str,
    #[serde(rename = "isReshareDisabledByAuthor")]
    pub is_reshare_disabled_by_author: bool,
    pub content: PostArticleContent<'a>,
}

#[derive(Debug, Serialize)]
pub struct PostArticleContent<'a> {
    pub article: PostArticleItem<'a>,
}

#[derive(Debug, Serialize)]
pub struct PostArticleItem<'a> {
    pub source: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct CommentBody<'a> {
    pub actor: &'a str,
    pub message: CommentMessage<'a>,
}

#[derive(Debug, Serialize)]
pub struct CommentMessage<'a> {
    pub text: &'a str,
}

#[derive(Debug, Serialize)]
pub struct ReactionBody<'a> {
    pub root: &'a str,
    #[serde(rename = "reactionType")]
    pub reaction_type: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Distribution {
    #[serde(rename = "feedDistribution")]
    pub feed_distribution: &'static str,                 // "MAIN_FEED" | "NONE"
    #[serde(rename = "targetEntities")]
    pub target_entities: Vec<String>,
    #[serde(rename = "thirdPartyDistributionChannels")]
    pub third_party_distribution_channels: Vec<String>,
}

impl Default for Distribution {
    fn default() -> Self {
        Self {
            feed_distribution: "MAIN_FEED",
            target_entities: vec![],
            third_party_distribution_channels: vec![],
        }
    }
}
