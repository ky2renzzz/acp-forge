//! Error types for web source integration.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("provider {name} failed: {reason}")]
    ProviderFailed { name: String, reason: String },

    #[error("no providers returned results for query: {query}")]
    NoResults { query: String },

    #[error("verification failed: only {confirmed}/{required} sources confirmed")]
    VerificationFailed { confirmed: usize, required: usize },

    #[error("http: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}
