//! Data transfer objects for Snipe-IT inventory resources.

use serde::Deserialize;
use serde::Serialize;

/// Data transfer object representing a physical or logical inventory location.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationDto {
    id: u64,
    name: String,
    #[serde(default)]
    parent_id: Option<u64>,
}

impl LocationDto {
    /// Creates a new location data transfer object.
    #[must_use]
    pub fn new(id: u64, name: String, parent_id: Option<u64>) -> Self {
        Self {
            id,
            name,
            parent_id,
        }
    }

    /// Returns the primary database identifier for the location.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the human-readable name of the location.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the identifier of the parent location, if one exists.
    #[must_use]
    pub const fn parent_id(&self) -> Option<u64> {
        self.parent_id
    }
}

/// Data transfer object representing an inventory asset tracked in Snipe-IT.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDto {
    id: u64,
    asset_tag: String,
    name: String,
    #[serde(default)]
    serial: Option<String>,
    #[serde(default)]
    location_id: Option<u64>,
}

impl AssetDto {
    /// Creates a new asset data transfer object.
    #[must_use]
    pub fn new(
        id: u64,
        asset_tag: String,
        name: String,
        serial: Option<String>,
        location_id: Option<u64>,
    ) -> Self {
        Self {
            id,
            asset_tag,
            name,
            serial,
            location_id,
        }
    }

    /// Returns the primary database identifier for the asset.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the asset tag string (e.g. "ASSET-00123").
    #[must_use]
    pub fn asset_tag(&self) -> &str {
        &self.asset_tag
    }

    /// Returns the asset name or model title.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the hardware serial number, if recorded.
    #[must_use]
    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    /// Returns the identifier of the assigned location, if one exists.
    #[must_use]
    pub const fn location_id(&self) -> Option<u64> {
        self.location_id
    }
}

/// Paginated list response wrapper matching Snipe-IT REST API schema.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    total: usize,
    rows: Vec<T>,
}

impl<T> PaginatedResponse<T> {
    /// Creates a new paginated response wrapper.
    #[must_use]
    pub fn new(total: usize, rows: Vec<T>) -> Self {
        Self { total, rows }
    }

    /// Returns the total count of matching records across all pages.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns a slice of the items returned in this page.
    #[must_use]
    pub fn rows(&self) -> &[T] {
        &self.rows
    }
}

impl<T> Default for PaginatedResponse<T> {
    fn default() -> Self {
        Self {
            total: 0,
            rows: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_dto_accessors_and_serialization() {
        let loc = LocationDto::new(101, "Bin Alpha".to_owned(), Some(42));
        assert_eq!(loc.id(), 101);
        assert_eq!(loc.name(), "Bin Alpha");
        assert_eq!(loc.parent_id(), Some(42));

        let json = serde_json::to_string(&loc).expect("serialization should succeed");
        let parsed: LocationDto =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(parsed, loc);
    }

    #[test]
    fn test_location_dto_deserialize_without_parent() {
        let json = r#"{"id":5,"name":"Root Building"}"#;
        let parsed: LocationDto = serde_json::from_str(json)
            .expect("deserialization with missing parent_id should succeed");
        assert_eq!(parsed.id(), 5);
        assert_eq!(parsed.name(), "Root Building");
        assert_eq!(parsed.parent_id(), None);
    }

    #[test]
    fn test_asset_dto_accessors_and_serialization() {
        let asset = AssetDto::new(
            202,
            "TAG-001".to_owned(),
            "Patch Panel".to_owned(),
            Some("SN-998877".to_owned()),
            Some(101),
        );
        assert_eq!(asset.id(), 202);
        assert_eq!(asset.asset_tag(), "TAG-001");
        assert_eq!(asset.name(), "Patch Panel");
        assert_eq!(asset.serial(), Some("SN-998877"));
        assert_eq!(asset.location_id(), Some(101));

        let json = serde_json::to_string(&asset).expect("serialization should succeed");
        let parsed: AssetDto = serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(parsed, asset);
    }

    #[test]
    fn test_asset_dto_deserialize_minimal() {
        let json = r#"{"id":303,"asset_tag":"TAG-MIN","name":"Crimper"}"#;
        let parsed: AssetDto =
            serde_json::from_str(json).expect("minimal asset deserialization should succeed");
        assert_eq!(parsed.id(), 303);
        assert_eq!(parsed.asset_tag(), "TAG-MIN");
        assert_eq!(parsed.name(), "Crimper");
        assert_eq!(parsed.serial(), None);
        assert_eq!(parsed.location_id(), None);
    }

    #[test]
    fn test_paginated_response() {
        let rows = vec![
            LocationDto::new(1, "L1".to_owned(), None),
            LocationDto::new(2, "L2".to_owned(), Some(1)),
        ];
        let paginated = PaginatedResponse::new(2, rows);
        assert_eq!(paginated.total(), 2);
        assert_eq!(paginated.rows().len(), 2);

        let json = serde_json::to_string(&paginated).expect("serialization should succeed");
        let parsed: PaginatedResponse<LocationDto> =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(parsed.total(), 2);
        assert_eq!(parsed.rows().len(), 2);
        assert_eq!(parsed.rows()[0].name(), "L1");
    }

    #[test]
    fn test_paginated_response_default() {
        let default_paginated: PaginatedResponse<AssetDto> = PaginatedResponse::default();
        assert_eq!(default_paginated.total(), 0);
        assert!(default_paginated.rows().is_empty());
    }
}
