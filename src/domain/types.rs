//! Newtypes for the domain's primitive-obsession-prone values. Each wraps a raw
//! primitive (i64, String) so the type system -- not convention -- prevents mixing up,
//! say, an `AssetId` and an arbitrary `i64`, or a `MarketSlug` and any old `String`.
//! `#[sqlx(transparent)]` lets these bind/fetch directly in queries as their inner
//! value; `#[serde(transparent)]` serializes them as that same bare value in JSON
//! responses, so callers see plain numbers/strings, not `{"0": ...}` wrapper objects.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct AssetId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct MarketSlug(pub String);

impl MarketSlug {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The response languages `/api/assets` and `/api/catalog` support. A closed enum
/// rather than a bare `String` -- an invalid or absent value can only become `En`,
/// never an arbitrary unchecked string reaching a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Es,
    Pt,
    Ja,
    Zh,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
            Lang::Pt => "pt",
            Lang::Ja => "ja",
            Lang::Zh => "zh",
        }
    }

    /// An absent or unrecognized value defaults to `En` rather than erroring -- a
    /// client that doesn't care about language shouldn't have to think about this.
    pub fn parse_or_default(value: Option<&str>) -> Self {
        match value {
            Some("es") => Lang::Es,
            Some("pt") => Lang::Pt,
            Some("ja") => Lang::Ja,
            Some("zh") => Lang::Zh,
            _ => Lang::En,
        }
    }
}

/// The plaintext value of an API key as presented in the `x-api-key` header. Never
/// stored or logged -- only ever hashed via `.hash()` before touching the database.
#[derive(Debug, Clone)]
pub struct ApiKeyPlaintext(pub String);

impl ApiKeyPlaintext {
    pub fn hash(&self) -> ApiKeyHash {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(self.0.as_bytes());
        ApiKeyHash(digest.iter().map(|b| format!("{b:02x}")).collect())
    }
}

/// The SHA-256 hex digest of an API key -- what's actually stored and compared in
/// `api_keys.key_hash`. A separate type from `ApiKeyPlaintext` so the two can never be
/// accidentally interchanged (e.g. comparing a plaintext key against a stored hash).
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct ApiKeyHash(pub String);

/// Validated, clamped limit/offset for a paginated query. Constructing one always
/// produces in-range values -- callers never see or need to re-check raw client input.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Pagination {
    pub fn new(limit: Option<i64>, offset: Option<i64>, default_limit: i64, max_limit: i64) -> Self {
        Self {
            limit: limit.unwrap_or(default_limit).clamp(1, max_limit),
            offset: offset.unwrap_or(0).max(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_defaults_to_english_for_unknown_or_absent() {
        assert_eq!(Lang::parse_or_default(None).code(), "en");
        assert_eq!(Lang::parse_or_default(Some("fr")).code(), "en");
        assert_eq!(Lang::parse_or_default(Some("")).code(), "en");
    }

    #[test]
    fn lang_parses_every_supported_code() {
        assert_eq!(Lang::parse_or_default(Some("es")).code(), "es");
        assert_eq!(Lang::parse_or_default(Some("pt")).code(), "pt");
        assert_eq!(Lang::parse_or_default(Some("ja")).code(), "ja");
        assert_eq!(Lang::parse_or_default(Some("zh")).code(), "zh");
    }

    #[test]
    fn pagination_clamps_out_of_range_values() {
        let p = Pagination::new(Some(-5), Some(-1), 50, 500);
        assert_eq!(p.limit, 1); // clamped up to the minimum of 1
        assert_eq!(p.offset, 0); // negative offset clamped to 0

        let p = Pagination::new(Some(99_999), None, 50, 500);
        assert_eq!(p.limit, 500); // clamped down to max_limit

        let p = Pagination::new(None, None, 50, 500);
        assert_eq!(p.limit, 50); // default applied
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn api_key_hash_is_deterministic_sha256_hex() {
        let key = ApiKeyPlaintext("ca_live_test123".to_string());
        let hash1 = key.hash();
        let hash2 = key.hash();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.0.len(), 64); // hex-encoded SHA-256 digest
        assert!(hash1.0.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_keys_hash_differently() {
        let a = ApiKeyPlaintext("key-a".to_string()).hash();
        let b = ApiKeyPlaintext("key-b".to_string()).hash();
        assert_ne!(a, b);
    }
}
