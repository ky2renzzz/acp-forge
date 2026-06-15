//! Response cache with TTL-based expiry.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::provider::SearchResult;

/// A cached set of results for a query.
#[derive(Debug, Clone)]
struct CacheEntry {
    results: Vec<SearchResult>,
    inserted_at: DateTime<Utc>,
}

/// In-memory cache for search results.
#[derive(Debug)]
pub struct Cache {
    entries: HashMap<String, CacheEntry>,
    ttl: Duration,
}

impl Cache {
    /// Create a cache with the given TTL.
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    /// Normalize a query into a cache key.
    fn key(query: &str) -> String {
        query.trim().to_lowercase()
    }

    /// Look up cached results. Returns `None` if absent or expired.
    #[must_use]
    pub fn get(&self, query: &str) -> Option<&[SearchResult]> {
        let entry = self.entries.get(&Self::key(query))?;
        let age = Utc::now()
            .signed_duration_since(entry.inserted_at)
            .to_std()
            .unwrap_or(Duration::MAX);

        if age > self.ttl {
            None
        } else {
            Some(&entry.results)
        }
    }

    /// Store results for a query.
    pub fn put(&mut self, query: &str, results: Vec<SearchResult>) {
        self.entries.insert(
            Self::key(query),
            CacheEntry {
                results,
                inserted_at: Utc::now(),
            },
        );
    }

    /// Evict all expired entries.
    pub fn evict_expired(&mut self) {
        let now = Utc::now();
        self.entries.retain(|_, entry| {
            now.signed_duration_since(entry.inserted_at)
                .to_std()
                .unwrap_or(Duration::MAX)
                <= self.ttl
        });
    }

    /// Number of (possibly expired) entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_result(title: &str) -> SearchResult {
        SearchResult {
            title: title.to_owned(),
            url: "https://example.com".into(),
            snippet: String::new(),
            source: "test".into(),
            fetched_at: Utc::now(),
            score: None,
        }
    }

    #[test]
    fn cache_hit_and_miss() {
        let mut cache = Cache::new(Duration::from_secs(60));
        assert!(cache.get("rust async").is_none());

        cache.put("Rust Async", vec![make_result("r1")]);
        // Normalised key should match.
        assert!(cache.get("rust async").is_some());
    }
}
