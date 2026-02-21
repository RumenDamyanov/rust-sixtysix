//! # sixtysix
//!
//! Minimal backend engine + HTTP API for the traditional 24-card trick-taking
//! game **Sixty-six** (AKA *Schnapsen* variant family).
//!
//! # Modules
//!
//! - [`engine`] — Core engine: `Game` trait, `Action`, `Session`, `Store` trait, `Engine`.
//! - [`store`] — In-memory store implementation.
//! - [`api`] — HTTP server (axum-based).
//! - [`game`] — Sixty-six game rules implementation.

pub mod api;
pub mod engine;
pub mod game;
pub mod store;
