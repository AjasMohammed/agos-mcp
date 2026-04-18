use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagesListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    pub max_results: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,
}

impl Default for MessagesListQuery {
    fn default() -> Self {
        Self {
            q: None,
            max_results: 100,
            page_token: None,
            label_ids: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagesListResp {
    pub messages: Option<Vec<MessageRef>>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageRef {
    pub id: String,
    pub thread_id: String,
}

pub enum MessageFormat {
    Full,
    Metadata,
    Minimal,
    Raw,
}
impl MessageFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Metadata => "metadata",
            Self::Minimal => "minimal",
            Self::Raw => "raw",
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    pub label_ids: Option<Vec<String>>,
    pub snippet: Option<String>,
    pub payload: Option<MessagePart>,
    pub raw: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePart {
    pub part_id: String,
    pub mime_type: String,
    pub filename: String,
    pub headers: Vec<MessagePartHeader>,
    pub body: Option<MessagePartBody>,
    pub parts: Option<Vec<MessagePart>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MessagePartHeader {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartBody {
    pub attachment_id: Option<String>,
    pub size: i32,
    pub data: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub snippet: String,
    pub messages: Option<Vec<Message>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LabelsListResp {
    pub labels: Option<Vec<Label>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub message_list_visibility: Option<String>,
    pub label_list_visibility: Option<String>,
    pub type_: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FiltersListResp {
    pub filter: Option<Vec<Filter>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Filter {
    pub id: String,
    pub criteria: FilterCriteria,
    pub action: FilterAction,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterCriteria {
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub query: Option<String>,
    pub negated_query: Option<String>,
    pub has_attachment: Option<bool>,
    pub exclude_chats: Option<bool>,
    pub size: Option<i32>,
    pub size_comparison: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterAction {
    pub add_label_ids: Option<Vec<String>>,
    pub remove_label_ids: Option<Vec<String>>,
    pub forward: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DraftsListResp {
    pub drafts: Option<Vec<DraftRef>>,
    pub next_page_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DraftRef {
    pub id: String,
    pub message: Option<Message>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub email_address: String,
    pub messages_total: i32,
    pub threads_total: i32,
    pub history_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Attachment {
    pub size: i32,
    pub data: String,
}
