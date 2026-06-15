//! Source provider trait and common types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// A single search result from any provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Human-readable title.
    pub title: String,
    /// URL of the source.
    pub url: String,
    /// Text snippet or summary.
    pub snippet: String,
    /// Which provider returned this.
    pub source: String,
    /// When this result was fetched.
    pub fetched_at: DateTime<Utc>,
    /// Relevance score (0.0–1.0), if the provider supplies one.
    pub score: Option<f64>,
}

/// A source provider that can search the web.
///
/// Implement this trait for each backend (generic web search, X/Twitter,
/// RSS feeds, documentation sites, etc.).
pub trait Provider: Send + Sync {
    /// Human-readable name (e.g. "duckduckgo", "x-api").
    fn name(&self) -> &str;

    /// Perform a search and return results.
    fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> impl std::future::Future<Output = Result<Vec<SearchResult>>> + Send;

    /// Fetch the full text of a URL (optional).
    fn fetch_text(
        &self,
        _url: &str,
    ) -> impl std::future::Future<Output = Result<String>> + Send {
        async {
            Err(crate::Error::ProviderFailed {
                name: "unknown".into(),
                reason: "fetch_text not supported".into(),
            })
        }
    }
}
