//! CryptoAlly API -- clean-architecture layers.
//!
//! - `domain`: entities, newtypes, and repository traits ("ports"). No sqlx, no axum.
//! - `application`: thin services orchestrating the ports. No sqlx, no axum.
//! - `infrastructure`: the real Postgres-backed repository implementations.
//! - `http`: axum router, handlers, auth middleware, error mapping.
//!
//! The dependency rule flows inward: `http` and `infrastructure` depend on `domain`
//! and `application`; `domain` depends on nothing else in this crate. Swapping
//! Postgres for something else means writing new `infrastructure` types against the
//! same `domain::ports` traits -- nothing in `application` or `http` would need to
//! change. The same property is what makes the handlers in `http` testable against
//! in-memory fakes instead of a live database (see `tests/`).

pub mod application;
pub mod domain;
pub mod http;
pub mod infrastructure;

// Re-exported at the crate root for any caller that just needs a raw pool handle
// without going through the full domain/application layering.
pub use infrastructure::postgres::pool::pool;
