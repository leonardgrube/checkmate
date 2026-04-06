use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChecklistMeta {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChecklistItem {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "codeLang")]
    pub code_lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "skipReason"
    )]
    pub skip_reason: Option<String>,
    pub checked: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Section {
    pub heading: String,
    pub items: Vec<ChecklistItem>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Checklist {
    pub slug: String,
    pub meta: ChecklistMeta,
    pub sections: Vec<Section>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChecklistSummary {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChecklistChangeKind {
    Created,
    Updated,
    Deleted,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistChangeEvent {
    pub kind: ChecklistChangeKind,
    pub slug: String,
}

#[derive(Serialize, Deserialize)]
pub struct RunMetadata {
    pub template: String,
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RunSummary {
    pub template: String,
    pub filename: String,
    pub title: String,
    pub date: String,
    pub notes: Option<String>,
    pub checked: usize,
    pub total: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Run {
    pub meta: RunMetadata,
    pub sections: Vec<Section>,
}
