# Wave 1 Work Orders (`WO-001` – `WO-005`)

## Universal Google Rust Style Rules (`rust-pro` GRS-001..GRS-012)
1. **GRS-003 (Leaf-Only Modules)**: `lib.rs` and `mod.rs` MUST ONLY contain module-level doc comments (`//!`), `pub mod ...;`, and `pub use ...;`. Zero `struct`, `enum`, `trait`, `const`, `type`, or `fn` definitions in `lib.rs` or `mod.rs`.
2. **GRS-001 (Single-Module `use` Granularity)**: Every `use` statement imports from a single module path with ZERO `::` inside `{ ... }` braces (e.g., `use std::fmt::{Display, Formatter};` is valid; `use std::{fmt, io};` is FORBIDDEN).
3. **GRS-002 (No Wildcard Imports)**: Never use `use foo::*;` except `use super::*;` inside `#[cfg(test)] mod tests`.
4. **GRS-010 (Zero `.unwrap()`)**: Never use `.unwrap()` in production code; propagate errors with `Result<T, E>` and `?`. In tests, use `.expect("descriptive reason")`.
5. **GRS-011 (Getter Naming)**: Omit `get_` prefix on getters (`fn width_pt(&self)`, not `fn get_width_pt(&self)`).
6. **GRS-012 (Line & Comment Width)**: Code lines <= 100 columns; standalone `//` and `///` comments <= 80 columns.
7. **`#[non_exhaustive]` & `#[must_use]`**: Apply `#[non_exhaustive]` on every `pub struct` and `pub enum` in library crates. Apply `#[must_use]` on pure constructors/accessors.
8. **Missing Docs & Debug**: Every `pub` item must have a `///` doc comment (`missing_docs = "warn"`, `clippy -D warnings` is enforced), including `# Errors` on any `pub fn` returning `Result`. Every `pub struct` and `pub enum` must derive `Debug`.

---

## `WO-001`: Workspace Architect
**Assigned Files (Create ONLY these 4 files)**:
1. `/home/console-admin/Github/Label-Agent/.gitignore`:
   Ignore `/target/`, `**/*.rs.bk`, `.env`, `.env.*`, `*.swp`, `/tmp/`, `.superpowers/`, `*.odt`, `*.lbx`, `*.pdf`, `*.bmp`, `*.png`.
2. `/home/console-admin/Github/Label-Agent/rustfmt.toml`:
   ```toml
   # Google Rust Style Guide — Stable Toolchain rustfmt.toml
   edition = "2021"
   max_width = 100
   tab_spaces = 4
   hard_tabs = false
   newline_style = "Unix"
   use_small_heuristics = "Default"
   use_field_init_shorthand = true
   use_try_shorthand = true
   reorder_imports = true
   reorder_modules = true
   remove_nested_parens = true
   ```
3. `/home/console-admin/Github/Label-Agent/.cargo/config.toml`:
   Configure `[build]` defaults and `[target.aarch64-unknown-linux-gnu]` comments/flags for Raspberry Pi 3B+ (`Cortex-A53`) cross-compilation.
4. `/home/console-admin/Github/Label-Agent/Cargo.toml`:
   ```toml
   [workspace]
   resolver = "2"
   members = [
       "crates/label-core",
       "crates/label-inventory",
       "crates/label-render",
       "crates/label-print",
       "crates/label-voice",
       "crates/label-agent-daemon",
   ]

   [workspace.package]
   version = "0.1.0"
   edition = "2021"
   rust-version = "1.75"
   license = "MIT OR Apache-2.0"
   authors = ["Mr-Molina"]
   repository = "https://github.com/Mr-Molina/Label-Agent"

   [workspace.dependencies]
   label-core = { path = "crates/label-core" }
   label-inventory = { path = "crates/label-inventory" }
   label-render = { path = "crates/label-render" }
   label-print = { path = "crates/label-print" }
   label-voice = { path = "crates/label-voice" }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   thiserror = "2.0"
   ureq = { version = "2.10", features = ["json"] }
   qrcode = { version = "0.14", default-features = false }
   lexopt = "0.3"

   [workspace.lints.rust]
   missing_docs = "warn"
   unsafe_op_in_unsafe_fn = "deny"
   static_mut_refs = "deny"
   unreachable_pub = "warn"
   missing_debug_implementations = "warn"

   [workspace.lints.clippy]
   borrowed_box = "deny"
   ptr_arg = "deny"
   should_implement_trait = "deny"
   new_without_default = "warn"
   wrong_self_convention = "deny"
   len_without_is_empty = "deny"
   exhaustive_enums = "warn"
   exhaustive_structs = "warn"
   wildcard_imports = "deny"
   ref_option = "warn"
   undocumented_unsafe_blocks = "deny"
   multiple_unsafe_ops_per_block = "warn"
   unwrap_used = "warn"
   dbg_macro = "warn"
   todo = "warn"
   needless_for_each = "warn"
   must_use_candidate = "warn"
   doc_markdown = "warn"
   missing_errors_doc = "warn"
   missing_panics_doc = "warn"
   missing_safety_doc = "deny"

   [profile.release]
   lto = "thin"
   codegen-units = 1
   strip = "symbols"
   ```

---

## `WO-002`: Docs & Governance Specialist
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/README.md`:
   - Include the exact Refined Mission Statement:
     > **Label-Agent** is an autonomous, 100% pure-Rust voice-driven inventory and physical labeling appliance. Designed to run headless with a sub-25 MiB memory footprint on edge ARM64 hardware (Raspberry Pi 3 Model B+), it captures natural-language voice commands, resolves intent against the `Inventory-Server` 6-level location and asset taxonomy, compiles high-density QR and category-indexed thermal labels (`Dymo 30256` 4"×2.31", `Brother P-Touch` `.lbx`, and `ODF/ODT`) strictly in RAM (`tmpfs`), and dispatches jobs directly to physical label printers. **Rust-only, Rust-always: zero garbage collection, zero heavy interpreters, and compile-time hardware/domain correctness.**
   - Include Mermaid architecture diagram, 6-crate workspace table (`label-core`, `label-inventory`, `label-render`, `label-print`, `label-voice`, `label-agent-daemon`), `Inventory-Server` 6-Level Location Taxonomy reference (`Building -> Floor -> Room -> Furniture -> SubDivision -> ContainerBox`), `BoxLabels` physical geometry (`4.0" x 2.3098"` Dymo 30256 & Brother `.lbx` `192x192` 1-bit BMP), Raspberry Pi 3B+ (`10.0.0.10` / `sbc-lyrion`) hardware specifications from `retired_raspberry_pi_dossier.md`, CLI usage examples, and Google Rust Style verification commands.
2. `/home/console-admin/Github/Label-Agent/LICENSE`:
   Standard MIT License (`Copyright (c) 2026 Mr-Molina`).
3. `/home/console-admin/Github/Label-Agent/docs/superpowers/plans/2026-11-07-label-agent-foundation.md`:
   Copy the full implementation plan from `/home/console-admin/.gemini/antigravity/brain/46bade93-cae0-45e4-b7a2-df798447e621/implementation_plan.md`.

---

## `WO-003`: Core Taxonomy Implementer (`crates/label-core` Part 1)
**Assigned Files (Create ONLY these 4 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-core/Cargo.toml`:
   Uses `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, `[dependencies]` (`serde.workspace = true`, `thiserror.workspace = true`), `[lints] workspace = true`.
2. `/home/console-admin/Github/Label-Agent/crates/label-core/src/lib.rs`:
   Pure manifest ONLY (`GRS-003`):
   ```rust
   //! Core domain types, 6-level location taxonomy, label geometry, and voice
   //! command intents for Label-Agent.

   pub mod error;
   pub mod intent;
   pub mod label_spec;
   pub mod taxonomy;

   pub use crate::error::CoreError;
   pub use crate::intent::{LabelFormat, PrintMode, VoiceCommandIntent};
   pub use crate::label_spec::{AssetLabelSpec, BoxLabelSpec, PageGeometry};
   pub use crate::taxonomy::{LocationNode, LocationPath, TaxonomyLevel};
   ```
3. `/home/console-admin/Github/Label-Agent/crates/label-core/src/error.rs`:
   Define `#[non_exhaustive] #[derive(Debug, Error, Clone, PartialEq, Eq)] pub enum CoreError` with variants:
   - `InvalidTaxonomyDepth { depth: u8 }`
   - `InvalidContainerDepth { expected: usize, actual: usize }`
   - `EmptyLabelIdentifier`
   - `EmptyBaseUrl`
   - `InvalidDimension { field: &'static str }`
   Ensure every variant and field has `///` doc comments!
4. `/home/console-admin/Github/Label-Agent/crates/label-core/src/taxonomy.rs`:
   Define:
   - `#[non_exhaustive] #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)] pub enum TaxonomyLevel` (`Building = 1`, `Floor = 2`, `Room = 3`, `Furniture = 4`, `SubDivision = 5`, `ContainerBox = 6`) with `pub fn from_depth(depth: u8) -> Result<Self, CoreError>` and `#[must_use] pub const fn depth(self) -> u8` and `#[must_use] pub const fn is_container_box(self) -> bool`.
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct LocationNode` (private fields `id: u64`, `name: String`, `parent_id: Option<u64>`, `level: TaxonomyLevel`; constructor `#[must_use] pub fn new(...) -> Self` and getters `id(&self) -> u64`, `name(&self) -> &str`, `parent_id(&self) -> Option<u64>`, `level(&self) -> TaxonomyLevel`).
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct LocationPath` (private `nodes: Vec<LocationNode>`; `pub fn from_container_lineage(nodes: Vec<LocationNode>) -> Result<Self, CoreError>` verifying `nodes.len() == 6` and levels `1..=6` in strict order; `#[must_use] pub fn nodes(&self) -> &[LocationNode]`, `#[must_use] pub fn breadcrumb(&self) -> String` joining with `" > "`, `#[must_use] pub fn container_code(&self) -> &str`).
   - Include unit tests in `#[cfg(test)] mod tests`.

---

## `WO-004`: Core Specs & Intent Implementer (`crates/label-core` Part 2)
**Assigned Files (Create ONLY these 2 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-core/src/label_spec.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)] pub struct PageGeometry` with private fields `width_in: f64`, `height_in: f64`, `margin_in: f64`, `qr_size_in: f64`, `icon_size_in: f64`.
   - `#[must_use] pub const fn dymo_30256() -> Self` (`width_in: 4.0`, `height_in: 2.3098`, `margin_in: 0.1701`, `qr_size_in: 0.7`, `icon_size_in: 0.7`).
   - Getters `width_in`, `height_in`, `margin_in`, `qr_size_in`, `icon_size_in`, `width_pt(&self) -> f64` (`self.width_in * 72.0`), `height_pt(&self) -> f64` (`self.height_in * 72.0`), `margin_pt(&self) -> f64` (`self.margin_in * 72.0`).
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] pub struct BoxLabelSpec` with private fields (`label_id: String`, `base_url: String`, `category_icon: String`, `location_path: LocationPath`, `items: Vec<String>`, `geometry: PageGeometry`).
   - `pub fn new(label_id: &str, base_url: &str, category_icon: &str, location_path: LocationPath, items: Vec<String>) -> Result<Self, CoreError>` (rejects empty `label_id` or `base_url`).
   - Getters: `label_id(&self) -> &str`, `base_url(&self) -> &str`, `category_icon(&self) -> &str`, `location_path(&self) -> &LocationPath`, `items(&self) -> &[String]`, `geometry(&self) -> PageGeometry`, `#[must_use] pub fn qr_payload_url(&self) -> String` (`format!("{}/{}", self.base_url.trim_end_matches('/'), self.label_id)`).
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct AssetLabelSpec` (`asset_tag: String`, `asset_name: String`, `serial: Option<String>`, `model_name: String`, `qr_url: String`) with `pub fn new(...) -> Result<Self, CoreError>` and getters.
   - Unit tests in `#[cfg(test)] mod tests`.
2. `/home/console-admin/Github/Label-Agent/crates/label-core/src/intent.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub enum LabelFormat { Dymo30256Pdf, BoxLabelsOdt, BrotherPtouchLbx }`
   - `#[non_exhaustive] #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub enum PrintMode { ExecutePrint, DryRun }` (replaces raw `bool` per Google Rust Style `§6.1`).
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub enum VoiceCommandIntent`:
     - `PrintBoxLabel { box_code: String, format: LabelFormat }`
     - `QueryBoxContents { box_code: String }`
     - `CheckOutAssetToBox { asset_tag: String, box_code: String }`
     - `RelocateBox { box_code: String, target_parent_code: String }`
   - Unit tests in `#[cfg(test)] mod tests`.

---

## `WO-005`: Deployment & CI Engineer
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/deploy/systemd/label-agent.service`:
   Hardened `systemd` service unit targeting Raspberry Pi 3 Model B+ (`10.0.0.10` / Debian 13 `trixie` `aarch64`):
   - `After=network-online.target cups.service ntpsec.service`
   - `Wants=network-online.target`
   - `User=console-admin`, `Group=console-admin`, `SupplementaryGroups=audio lp`
   - `ExecStart=/usr/local/bin/label-agent --daemon --spool-dir /tmp/label-agent --printer Dymo_LabelWriter_450`
   - `MemoryMax=256M`, `MemoryHigh=192M`, `CPUQuota=350%`, `PrivateTmp=no` (uses `/tmp` `tmpfs` mounted on Pi 3B+), `ProtectSystem=strict`, `ReadWritePaths=/tmp`
2. `/home/console-admin/Github/Label-Agent/deploy/pi3b-prep-checklist.md`:
   Complete operational runbook for deploying `label-agent` onto `10.0.0.10` (`sbc-lyrion` -> `sbc-label-agent`), referencing `retired_raspberry_pi_dossier.md`:
   - Step 1: Purge dormant `lyrionmusicserver` & `nginx` (`+1.2 GB` reclaimed on `/dev/mmcblk0p2`).
   - Step 2: Unmask and enable `ntpsec.service` (mandatory for HTTPS/TLS to `Inventory-Server`) and `cups.socket` / `cups.service` (for `Dymo_LabelWriter_450`).
   - Step 3: Set hostname `sbc-label-agent` (`hostnamectl set-hostname sbc-label-agent`).
   - Step 4: Cross-compile (`cargo build --release --target aarch64-unknown-linux-gnu`) and deploy binary + `label-agent.service`.
3. `/home/console-admin/Github/Label-Agent/.github/workflows/ci.yml`:
   GitHub Actions workflow running on `push` and `pull_request` to `main`:
   - `cargo fmt --all -- --check`
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace --all-targets`
   - Cross-target check for `aarch64-unknown-linux-gnu` (`rustup target add aarch64-unknown-linux-gnu && cargo check --workspace --target aarch64-unknown-linux-gnu`).
