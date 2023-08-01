use serde::{Deserialize, Serialize};
use warp::ws::Message;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoteDAG {
    pub file_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoteDAGWrite {
    pub file_path: String,
    pub contents: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub file_name: String,
    pub file_path: String,
    pub is_dir: bool,
    /// in bytes
    pub size: u64,
    pub modified: u128,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOptions {
    pub file_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCell {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCellUpdate {
    pub id: String,
    pub name: String,
    pub value: String,
}

impl Into<Message> for RunCellUpdate {
    fn into(self) -> Message {
        let new_msg = serde_json::to_string(&self).unwrap();
        Message::text(new_msg)
    }
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelUpdate {
    pub status: String,
}
