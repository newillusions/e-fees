//! Domain model types for e-fees.
//!
//! All entity types, conversion helpers, and supporting structures
//! used across the e-fees ecosystem (Tauri desktop app, API service, etc.).

pub mod common;
pub mod project;
pub mod fee;
pub mod company;
pub mod contact;
pub mod activity;

pub use common::*;
pub use project::*;
pub use fee::*;
pub use company::*;
pub use contact::*;
pub use activity::*;
