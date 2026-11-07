//! Snipe-IT REST API client and 6-level location resolver for Label-Agent.

pub mod client;
pub mod error;
pub mod models;

pub use crate::client::{InventoryConfig, InventoryRepository, SnipeItClient};
pub use crate::error::InventoryError;
pub use crate::models::{AssetDto, LocationDto, PaginatedResponse};
