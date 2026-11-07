//! 6-level location taxonomy and hierarchical path representations.
//!
//! Provides types for modeling the 6-level location hierarchy of
//! Inventory-Server:
//! Building -> Floor -> Room -> Furniture -> `SubDivision` -> `ContainerBox`.

use crate::error::CoreError;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Formatter;

/// The 6 canonical hierarchy levels defined by Inventory-Server.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaxonomyLevel {
    /// Level 1: Building (e.g. Headquarters, Warehouse).
    Building = 1,
    /// Level 2: Floor (e.g. First Floor, Basement).
    Floor = 2,
    /// Level 3: Room (e.g. Server Room, Lab 101).
    Room = 3,
    /// Level 4: Furniture (e.g. Rack A, Workbench 3).
    Furniture = 4,
    /// Level 5: `SubDivision` (e.g. Shelf 2, Drawer B).
    SubDivision = 5,
    /// Level 6: `ContainerBox` (e.g. BOX-042, Tote 17).
    ContainerBox = 6,
}

impl TaxonomyLevel {
    /// Resolves a taxonomy level from an integer depth between 1 and 6.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidTaxonomyDepth`] if `depth` is not in 1..=6.
    pub fn from_depth(depth: u8) -> Result<Self, CoreError> {
        match depth {
            1 => Ok(Self::Building),
            2 => Ok(Self::Floor),
            3 => Ok(Self::Room),
            4 => Ok(Self::Furniture),
            5 => Ok(Self::SubDivision),
            6 => Ok(Self::ContainerBox),
            _ => Err(CoreError::InvalidTaxonomyDepth { depth }),
        }
    }

    /// Returns the numeric depth associated with this taxonomy level (1 to 6).
    #[must_use]
    pub const fn depth(self) -> u8 {
        match self {
            Self::Building => 1,
            Self::Floor => 2,
            Self::Room => 3,
            Self::Furniture => 4,
            Self::SubDivision => 5,
            Self::ContainerBox => 6,
        }
    }

    /// Returns `true` if this level represents a terminal container box.
    #[must_use]
    pub const fn is_container_box(self) -> bool {
        matches!(self, Self::ContainerBox)
    }
}

impl Display for TaxonomyLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Building => "Building",
            Self::Floor => "Floor",
            Self::Room => "Room",
            Self::Furniture => "Furniture",
            Self::SubDivision => "SubDivision",
            Self::ContainerBox => "ContainerBox",
        };
        f.write_str(name)
    }
}

/// A node within the location taxonomy tree.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationNode {
    id: u64,
    name: String,
    parent_id: Option<u64>,
    level: TaxonomyLevel,
}

impl LocationNode {
    /// Creates a new location node.
    #[must_use]
    pub fn new(
        id: u64,
        name: impl Into<String>,
        parent_id: Option<u64>,
        level: TaxonomyLevel,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            parent_id,
            level,
        }
    }

    /// Returns the database ID of this location node.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the name or identifier of this location node.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parent node ID, if one exists.
    #[must_use]
    pub const fn parent_id(&self) -> Option<u64> {
        self.parent_id
    }

    /// Returns the taxonomy level of this location node.
    #[must_use]
    pub const fn level(&self) -> TaxonomyLevel {
        self.level
    }
}

/// A verified 6-level hierarchical path from Building down to `ContainerBox`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationPath {
    nodes: Vec<LocationNode>,
}

impl LocationPath {
    /// Constructs a verified 6-level location path from a container's lineage.
    ///
    /// # Errors
    ///
    /// - Returns [`CoreError::InvalidContainerDepth`] if `nodes.len() != 6`.
    /// - Returns [`CoreError::InvalidTaxonomyDepth`] if the nodes do not match
    ///   levels 1 through 6 in strict sequential order.
    pub fn from_container_lineage(nodes: Vec<LocationNode>) -> Result<Self, CoreError> {
        if nodes.len() != 6 {
            return Err(CoreError::InvalidContainerDepth {
                expected: 6,
                actual: nodes.len(),
            });
        }

        for (index, node) in nodes.iter().enumerate() {
            let expected_depth = (index + 1) as u8;
            if node.level().depth() != expected_depth {
                return Err(CoreError::InvalidTaxonomyDepth {
                    depth: node.level().depth(),
                });
            }
        }

        Ok(Self { nodes })
    }

    /// Returns the slice of 6 location nodes comprising this path.
    #[must_use]
    pub fn nodes(&self) -> &[LocationNode] {
        &self.nodes
    }

    /// Formats the full path hierarchy as a human-readable breadcrumb string
    /// joined by `" > "` (e.g. `"HQ > Floor 2 > Lab > Rack A > Shelf 1 > Box 42"`).
    #[must_use]
    pub fn breadcrumb(&self) -> String {
        self.nodes
            .iter()
            .map(LocationNode::name)
            .collect::<Vec<_>>()
            .join(" > ")
    }

    /// Returns the container code or name of the terminal container box.
    ///
    /// Returns an empty string if the node list is unexpectedly empty.
    #[must_use]
    pub fn container_code(&self) -> &str {
        self.nodes.last().map_or("", LocationNode::name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lineage() -> Vec<LocationNode> {
        vec![
            LocationNode::new(1, "Main Building", None, TaxonomyLevel::Building),
            LocationNode::new(2, "Floor 2", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "Hardware Lab", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "Rack Alpha", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "Bin Array 4", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, "BOX-042", Some(5), TaxonomyLevel::ContainerBox),
        ]
    }

    #[test]
    fn test_taxonomy_level_from_depth_valid() {
        let cases = [
            (1, TaxonomyLevel::Building),
            (2, TaxonomyLevel::Floor),
            (3, TaxonomyLevel::Room),
            (4, TaxonomyLevel::Furniture),
            (5, TaxonomyLevel::SubDivision),
            (6, TaxonomyLevel::ContainerBox),
        ];

        for (depth, expected) in cases {
            let level = TaxonomyLevel::from_depth(depth).expect("valid depth must resolve");
            assert_eq!(level, expected);
            assert_eq!(level.depth(), depth);
        }

        assert!(!TaxonomyLevel::Building.is_container_box());
        assert!(!TaxonomyLevel::Floor.is_container_box());
        assert!(!TaxonomyLevel::Room.is_container_box());
        assert!(!TaxonomyLevel::Furniture.is_container_box());
        assert!(!TaxonomyLevel::SubDivision.is_container_box());
        assert!(TaxonomyLevel::ContainerBox.is_container_box());
    }

    #[test]
    fn test_taxonomy_level_from_depth_invalid() {
        assert_eq!(
            TaxonomyLevel::from_depth(0),
            Err(CoreError::InvalidTaxonomyDepth { depth: 0 })
        );
        assert_eq!(
            TaxonomyLevel::from_depth(7),
            Err(CoreError::InvalidTaxonomyDepth { depth: 7 })
        );
        assert_eq!(
            TaxonomyLevel::from_depth(255),
            Err(CoreError::InvalidTaxonomyDepth { depth: 255 })
        );
    }

    #[test]
    fn test_taxonomy_level_display() {
        assert_eq!(TaxonomyLevel::Building.to_string(), "Building");
        assert_eq!(TaxonomyLevel::ContainerBox.to_string(), "ContainerBox");
    }

    #[test]
    fn test_location_node_accessors() {
        let node = LocationNode::new(42, "Storage Closet", Some(10), TaxonomyLevel::Room);
        assert_eq!(node.id(), 42);
        assert_eq!(node.name(), "Storage Closet");
        assert_eq!(node.parent_id(), Some(10));
        assert_eq!(node.level(), TaxonomyLevel::Room);
    }

    #[test]
    fn test_location_path_from_valid_lineage() {
        let lineage = sample_lineage();
        let path =
            LocationPath::from_container_lineage(lineage).expect("valid lineage must succeed");

        assert_eq!(path.nodes().len(), 6);
        assert_eq!(
            path.breadcrumb(),
            "Main Building > Floor 2 > Hardware Lab > Rack Alpha > Bin Array 4 > BOX-042"
        );
        assert_eq!(path.container_code(), "BOX-042");
    }

    #[test]
    fn test_location_path_invalid_length() {
        let mut short_lineage = sample_lineage();
        short_lineage.pop();
        let err =
            LocationPath::from_container_lineage(short_lineage).expect_err("5 nodes must fail");
        assert_eq!(
            err,
            CoreError::InvalidContainerDepth {
                expected: 6,
                actual: 5,
            }
        );

        let mut long_lineage = sample_lineage();
        long_lineage.push(LocationNode::new(
            7,
            "Extra SubBox",
            Some(6),
            TaxonomyLevel::ContainerBox,
        ));
        let err =
            LocationPath::from_container_lineage(long_lineage).expect_err("7 nodes must fail");
        assert_eq!(
            err,
            CoreError::InvalidContainerDepth {
                expected: 6,
                actual: 7,
            }
        );
    }

    #[test]
    fn test_location_path_invalid_sequence() {
        let mut out_of_order = sample_lineage();
        // Swap Floor and Room
        out_of_order[1] = LocationNode::new(2, "Wrong Room", Some(1), TaxonomyLevel::Room);
        let err = LocationPath::from_container_lineage(out_of_order)
            .expect_err("out of order levels must fail");
        assert_eq!(err, CoreError::InvalidTaxonomyDepth { depth: 3 });
    }
}
