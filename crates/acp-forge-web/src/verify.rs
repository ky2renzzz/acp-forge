//! Cross-reference verification.
//!
//! Given a claim (a short factual statement), query multiple providers
//! and check whether ≥ N sources corroborate it. This is a heuristic —
//! not a proof — but it catches obvious hallucinations.

use crate::cache::Cache;
use crate::error::Result;
use crate::provider::{Provider, SearchResult};

/// Outcome of a verification check.
#[derive(Debug)]
pub struct Verification {
    /// The original claim.
    pub claim: String,
    /// How many distinct sources mentioned the claim.
    pub confirmed_by: usize,
    /// Minimum required for the claim to pass.
    pub required: usize,
    /// Whether the claim passed.
    pub passed: bool,
    /// The supporting results.
    pub evidence: Vec<SearchResult>,
}

/// Verify a claim against multiple providers.
///
/// `min_sources` is the number of independent sources that must
/// mention the claim for it to be considered verified.
pub async fn verify_claim<P: Provider>(
    claim: &str,
    providers: &[P],
    cache: &mut Cache,
    min_sources: usize,
) -> Result<Verification> {
    let mut all_results: Vec<SearchResult> = Vec::new();

    // Check cache first.
    if let Some(cached) = cache.get(claim) {
        all_results.extend(cached.iter().cloned());
    } else {
        // Query all providers.
        for provider in providers {
            match provider.search(claim, 5).await {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    tracing::warn!(
                        provider = provider.name(),
                        error = %e,
                        "provider failed during verification"
                    );
                }
            }
        }
        cache.put(claim, all_results.clone());
    }

    // Count distinct sources (by domain).
    let mut domains = std::collections::HashSet::new();
    let claim_lower = claim.to_lowercase();

    let evidence: Vec<SearchResult> = all_results
        .into_iter()
        .filter(|r| {
            let text = format!("{} {}", r.title, r.snippet).to_lowercase();
            // Simple substring containment. A production system would
            // use embeddings or NLI.
            text.contains(&claim_lower)
                || claim_lower
                    .split_whitespace()
                    .filter(|w| w.len() > 3)
                    .all(|word| text.contains(word))
        })
        .filter(|r| {
            // Deduplicate by domain.
            extract_domain(&r.url)
                .map(|d| domains.insert(d))
                .unwrap_or(true)
        })
        .collect();

    let confirmed_by = evidence.len();
    let passed = confirmed_by >= min_sources;

    Ok(Verification {
        claim: claim.to_owned(),
        confirmed_by,
        required: min_sources,
        passed,
        evidence,
    })
}

/// Extract the domain from a URL for deduplication.
fn extract_domain(url: &str) -> Option<String> {
    let start = url.find("://").map(|i| i + 3).unwrap_or(0);
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let domain = &rest[..end];
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_domain_works() {
        assert_eq!(
            extract_domain("https://docs.rs/tokio/latest"),
            Some("docs.rs".into())
        );
        assert_eq!(
            extract_domain("http://example.com/path"),
            Some("example.com".into())
        );
    }
}
