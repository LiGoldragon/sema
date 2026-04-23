//! sema — the content-addressed record database.
//!
//! Pseudo-sema for now: rkyv-typed Rust records stored in redb,
//! addressed by blake3 hash. Each record is a sealed unit pointing
//! at other sealed units by content hash. Opera — the database's
//! compilation-unit concept — are collections of records rooted at
//! a module.
//!
//! The full sema format (universal typed-binary of meaning,
//! self-transforming, quorum-signed) follows from this foundation.
