//! Physical label specifications, dimension geometries, and payload schemas.
//!
//! Models target print geometries (such as Dymo 30256 thermal labels), full
//! container box label specifications (including taxonomy paths and items),
//! and individual asset tag label specifications.

use crate::error::CoreError;
use crate::taxonomy::LocationPath;
use serde::Deserialize;
use serde::Serialize;

/// Physical dimension geometry and margins for label layout calculation.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageGeometry {
    width_in: f64,
    height_in: f64,
    margin_in: f64,
    qr_size_in: f64,
    icon_size_in: f64,
}

impl PageGeometry {
    /// Creates a custom page geometry specification.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidDimension`] if any dimension is non-positive
    /// or if margins are negative.
    pub fn new(
        width_in: f64,
        height_in: f64,
        margin_in: f64,
        qr_size_in: f64,
        icon_size_in: f64,
    ) -> Result<Self, CoreError> {
        if width_in <= 0.0 {
            return Err(CoreError::InvalidDimension { field: "width_in" });
        }
        if height_in <= 0.0 {
            return Err(CoreError::InvalidDimension { field: "height_in" });
        }
        if margin_in < 0.0 {
            return Err(CoreError::InvalidDimension { field: "margin_in" });
        }
        if qr_size_in <= 0.0 {
            return Err(CoreError::InvalidDimension {
                field: "qr_size_in",
            });
        }
        if icon_size_in <= 0.0 {
            return Err(CoreError::InvalidDimension {
                field: "icon_size_in",
            });
        }
        Ok(Self {
            width_in,
            height_in,
            margin_in,
            qr_size_in,
            icon_size_in,
        })
    }

    /// Default geometry for standard Dymo 30256 Large Address labels
    /// (4" x 2.31").
    #[must_use]
    pub const fn dymo_30256() -> Self {
        Self {
            width_in: 4.0,
            height_in: 2.3098,
            margin_in: 0.1701,
            qr_size_in: 0.7,
            icon_size_in: 0.7,
        }
    }

    /// Returns the page width in inches.
    #[must_use]
    pub const fn width_in(&self) -> f64 {
        self.width_in
    }

    /// Returns the page height in inches.
    #[must_use]
    pub const fn height_in(&self) -> f64 {
        self.height_in
    }

    /// Returns the page border margin in inches.
    #[must_use]
    pub const fn margin_in(&self) -> f64 {
        self.margin_in
    }

    /// Returns the square QR code dimension in inches.
    #[must_use]
    pub const fn qr_size_in(&self) -> f64 {
        self.qr_size_in
    }

    /// Returns the square category icon dimension in inches.
    #[must_use]
    pub const fn icon_size_in(&self) -> f64 {
        self.icon_size_in
    }

    /// Computes page width in PostScript points (1 inch = 72 points).
    #[must_use]
    pub fn width_pt(&self) -> f64 {
        self.width_in * 72.0
    }

    /// Computes page height in PostScript points (1 inch = 72 points).
    #[must_use]
    pub fn height_pt(&self) -> f64 {
        self.height_in * 72.0
    }

    /// Computes margin in PostScript points (1 inch = 72 points).
    #[must_use]
    pub fn margin_pt(&self) -> f64 {
        self.margin_in * 72.0
    }
}

impl Default for PageGeometry {
    fn default() -> Self {
        Self::dymo_30256()
    }
}

/// Complete specification required to render and print a container box label.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoxLabelSpec {
    label_id: String,
    base_url: String,
    category_icon: String,
    location_path: LocationPath,
    items: Vec<String>,
    geometry: PageGeometry,
}

impl BoxLabelSpec {
    /// Constructs a new container box label specification.
    ///
    /// # Errors
    ///
    /// - Returns [`CoreError::EmptyLabelIdentifier`] if `label_id` is empty.
    /// - Returns [`CoreError::EmptyBaseUrl`] if `base_url` is empty.
    pub fn new(
        label_id: &str,
        base_url: &str,
        category_icon: &str,
        location_path: LocationPath,
        items: Vec<String>,
    ) -> Result<Self, CoreError> {
        if label_id.trim().is_empty() {
            return Err(CoreError::EmptyLabelIdentifier);
        }
        if base_url.trim().is_empty() {
            return Err(CoreError::EmptyBaseUrl);
        }

        Ok(Self {
            label_id: label_id.to_string(),
            base_url: base_url.to_string(),
            category_icon: category_icon.to_string(),
            location_path,
            items,
            geometry: PageGeometry::dymo_30256(),
        })
    }

    /// Overrides the default label geometry.
    #[must_use]
    pub fn with_geometry(mut self, geometry: PageGeometry) -> Self {
        self.geometry = geometry;
        self
    }

    /// Returns the unique container box identifier string.
    #[must_use]
    pub fn label_id(&self) -> &str {
        &self.label_id
    }

    /// Returns the inventory base URL prefix.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the category icon name or symbol identifier.
    #[must_use]
    pub fn category_icon(&self) -> &str {
        &self.category_icon
    }

    /// Returns the verified 6-level taxonomy hierarchy for this box.
    #[must_use]
    pub fn location_path(&self) -> &LocationPath {
        &self.location_path
    }

    /// Returns the list of item descriptions or asset names inside the box.
    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Returns the physical page layout geometry.
    #[must_use]
    pub fn geometry(&self) -> PageGeometry {
        self.geometry
    }

    /// Computes the full URL encoded into the box's QR code.
    #[must_use]
    pub fn qr_payload_url(&self) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), self.label_id)
    }
}

/// Specification for an individual equipment or component asset tag label.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetLabelSpec {
    asset_tag: String,
    asset_name: String,
    serial: Option<String>,
    model_name: String,
    qr_url: String,
}

impl AssetLabelSpec {
    /// Constructs a new asset tag label specification.
    ///
    /// # Errors
    ///
    /// - Returns [`CoreError::EmptyLabelIdentifier`] if `asset_tag` is empty.
    /// - Returns [`CoreError::EmptyBaseUrl`] if `qr_url` is empty.
    pub fn new(
        asset_tag: &str,
        asset_name: &str,
        serial: Option<&str>,
        model_name: &str,
        qr_url: &str,
    ) -> Result<Self, CoreError> {
        if asset_tag.trim().is_empty() {
            return Err(CoreError::EmptyLabelIdentifier);
        }
        if qr_url.trim().is_empty() {
            return Err(CoreError::EmptyBaseUrl);
        }

        Ok(Self {
            asset_tag: asset_tag.to_string(),
            asset_name: asset_name.to_string(),
            serial: serial.map(str::to_string),
            model_name: model_name.to_string(),
            qr_url: qr_url.to_string(),
        })
    }

    /// Returns the inventory tracking tag (e.g. "AST-0042").
    #[must_use]
    pub fn asset_tag(&self) -> &str {
        &self.asset_tag
    }

    /// Returns the common asset display name.
    #[must_use]
    pub fn asset_name(&self) -> &str {
        &self.asset_name
    }

    /// Returns the optional hardware serial number.
    #[must_use]
    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    /// Returns the model name or manufacturer specification.
    #[must_use]
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Returns the direct URL destination encoded in the asset QR code.
    #[must_use]
    pub fn qr_url(&self) -> &str {
        &self.qr_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taxonomy::LocationNode;
    use crate::taxonomy::TaxonomyLevel;

    fn make_test_location_path() -> LocationPath {
        let nodes = vec![
            LocationNode::new(1, "Main Campus", None, TaxonomyLevel::Building),
            LocationNode::new(2, "Floor 1", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "Room 101", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "Rack A", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "Shelf 2", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, "BOX-001-Wood", Some(5), TaxonomyLevel::ContainerBox),
        ];
        LocationPath::from_container_lineage(nodes).expect("test lineage must be valid 6 levels")
    }

    #[test]
    fn test_page_geometry_dymo_30256_constants() {
        let geo = PageGeometry::dymo_30256();
        assert!((geo.width_in() - 4.0).abs() < f64::EPSILON);
        assert!((geo.height_in() - 2.3098).abs() < f64::EPSILON);
        assert!((geo.margin_in() - 0.1701).abs() < f64::EPSILON);
        assert!((geo.qr_size_in() - 0.7).abs() < f64::EPSILON);
        assert!((geo.icon_size_in() - 0.7).abs() < f64::EPSILON);

        assert!((geo.width_pt() - 288.0).abs() < 1e-6);
        assert!((geo.height_pt() - 166.3056).abs() < 1e-4);
        assert!((geo.margin_pt() - 12.2472).abs() < 1e-4);
    }

    #[test]
    fn test_page_geometry_custom_and_validation() {
        let geo = PageGeometry::new(3.0, 2.0, 0.1, 0.5, 0.5).expect("valid custom geometry");
        assert!((geo.width_in() - 3.0).abs() < f64::EPSILON);

        assert!(matches!(
            PageGeometry::new(0.0, 2.0, 0.1, 0.5, 0.5),
            Err(CoreError::InvalidDimension { field: "width_in" })
        ));
        assert!(matches!(
            PageGeometry::new(3.0, -1.0, 0.1, 0.5, 0.5),
            Err(CoreError::InvalidDimension { field: "height_in" })
        ));
    }

    #[test]
    fn test_box_label_spec_creation_and_payload() {
        let path = make_test_location_path();
        let items = vec!["M3 Screws".to_string(), "Washers".to_string()];
        let spec = BoxLabelSpec::new(
            "BOX-001-Wood",
            "https://inventory.example.com",
            "hardware-box",
            path,
            items.clone(),
        )
        .expect("box label spec must succeed");

        assert_eq!(spec.label_id(), "BOX-001-Wood");
        assert_eq!(spec.base_url(), "https://inventory.example.com");
        assert_eq!(spec.category_icon(), "hardware-box");
        assert_eq!(spec.items(), items.as_slice());
        assert_eq!(
            spec.qr_payload_url(),
            "https://inventory.example.com/BOX-001-Wood"
        );
        assert_eq!(
            spec.location_path().breadcrumb(),
            "Main Campus > Floor 1 > Room 101 > Rack A > Shelf 2 > BOX-001-Wood"
        );
    }

    #[test]
    fn test_box_label_spec_validation() {
        let path = make_test_location_path();
        assert!(matches!(
            BoxLabelSpec::new("", "https://example.com", "icon", path.clone(), vec![]),
            Err(CoreError::EmptyLabelIdentifier)
        ));
        assert!(matches!(
            BoxLabelSpec::new("BOX-1", "   ", "icon", path, vec![]),
            Err(CoreError::EmptyBaseUrl)
        ));
    }

    #[test]
    fn test_asset_label_spec_creation_and_validation() {
        let spec = AssetLabelSpec::new(
            "AST-100",
            "Oscilloscope 100MHz",
            Some("SN-998822"),
            "Rigol DS1054Z",
            "https://inventory.example.com/hardware/100",
        )
        .expect("asset spec must succeed");

        assert_eq!(spec.asset_tag(), "AST-100");
        assert_eq!(spec.asset_name(), "Oscilloscope 100MHz");
        assert_eq!(spec.serial(), Some("SN-998822"));
        assert_eq!(spec.model_name(), "Rigol DS1054Z");
        assert_eq!(spec.qr_url(), "https://inventory.example.com/hardware/100");

        assert!(matches!(
            AssetLabelSpec::new("", "Oscilloscope", None, "Rigol", "https://example.com"),
            Err(CoreError::EmptyLabelIdentifier)
        ));
        assert!(matches!(
            AssetLabelSpec::new("AST-1", "Oscilloscope", None, "Rigol", ""),
            Err(CoreError::EmptyBaseUrl)
        ));
    }
}
