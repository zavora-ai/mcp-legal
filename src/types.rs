use serde::{Deserialize, Serialize};

/// Normalized legal reference response — every tool returns this shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalResult {
    pub source: String,
    pub source_type: String,
    pub jurisdiction: String,
    pub title: String,
    pub citation: Option<String>,
    pub source_url: Option<String>,
    pub retrieved_at: String,
    pub published_at: Option<String>,
    pub effective_date: Option<String>,
    pub version_status: String,
    pub text: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub not_legal_advice: bool,
    pub human_review_recommended: bool,
}

/// Sanctions match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsResult {
    pub source: String,
    pub entity_name: String,
    pub match_confidence: Option<f64>,
    pub matched_fields: Vec<String>,
    pub aliases: Vec<String>,
    pub list_authority: Option<String>,
    pub datasets: Vec<String>,
    pub date_listed: Option<String>,
    pub source_url: Option<String>,
    pub retrieved_at: String,
    pub not_legal_advice: bool,
    pub human_review_recommended: bool,
}

/// Jurisdiction coverage info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionInfo {
    pub jurisdiction: String,
    pub sources: Vec<SourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub name: String,
    pub data_types: Vec<String>,
    pub update_cadence: String,
    pub reliability: String,
}
