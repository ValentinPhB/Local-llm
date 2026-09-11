//! Public JSON contracts only. No key, policy path or authoritative identity.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayIdentity {
    pub id: String,
    pub display_name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub authenticated: bool,
    pub identity: DisplayIdentity,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentResponse {
    pub resource_id: String,
    pub classification: String,
    pub content: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub resource_id: String,
    pub classification: String,
    pub excerpt: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
