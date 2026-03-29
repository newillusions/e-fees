//! Domain model types for e-fees.
//!
//! All entity types, conversion helpers, and supporting structures
//! used across the e-fees ecosystem (Tauri desktop app, API service, etc.).

pub mod activity;
pub mod common;
pub mod company;
pub mod contact;
pub mod fee;
pub mod project;

pub use activity::*;
pub use common::*;
pub use company::*;
pub use contact::*;
pub use fee::*;
pub use project::*;
