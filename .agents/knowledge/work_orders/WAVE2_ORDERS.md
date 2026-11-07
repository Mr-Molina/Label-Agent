# Wave 2 Work Orders (`WO-006` – `WO-013b`)

## Universal Google Rust Style Rules (`rust-pro` GRS-001..GRS-012)
1. **GRS-003 (Leaf-Only Modules)**: `lib.rs` and `mod.rs` MUST ONLY contain `//!` docs, `pub mod ...;`, and `pub use ...;`. Zero `struct`, `enum`, `trait`, `const`, `type`, or `fn` definitions in `lib.rs` or `mod.rs`.
2. **GRS-001 (Single-Module `use` Granularity)**: Every `use` statement imports from a single module path with ZERO `::` inside `{ ... }` braces.
3. **GRS-002 (No Wildcard Imports)**: Never use `use foo::*;` except `use super::*;` inside `#[cfg(test)] mod tests`.
4. **GRS-009 (No Combinator Chains in `for` Headers or `.for_each()`)**: Use plain `for` loops.
5. **GRS-010 (Zero `.unwrap()`)**: Never use `.unwrap()` in production code; propagate errors with `Result<T, E>` and `?`. In tests, use `.expect("descriptive reason")`.
6. **GRS-011 (Getter Naming & Mutex Access)**: Omit `get_` prefix on getters. For `Mutex<T>` fields, expose closure accessors (`with_foo`) and NEVER return `MutexGuard` or call `.into_inner()` on a `PoisonError` (`GRS-007`).
7. **GRS-012 (Line & Comment Width)**: Code lines <= 100 columns; standalone `//` and `///` comments <= 80 columns.
8. **`#[non_exhaustive]` & `#[must_use]`**: Apply `#[non_exhaustive]` on every `pub struct` and `pub enum` in library crates. Apply `#[must_use]` on pure constructors/accessors.
9. **Missing Docs & Debug**: Every `pub` item must have a `///` doc comment (`missing_docs = "warn"`), including `# Errors` on any `pub fn` returning `Result` and `# Panics` if `.expect()` is used. Every `pub struct` and `pub enum` must derive `Debug`.

---

## `WO-006`: Inventory Manifest & Error (`crates/label-inventory` Part A)
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-inventory/Cargo.toml`:
   Workspace metadata (`version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, `rust-version.workspace = true`, `authors.workspace = true`, `repository.workspace = true`), `[dependencies]` (`label-core.workspace = true`, `serde.workspace = true`, `serde_json.workspace = true`, `thiserror.workspace = true`, `ureq.workspace = true`), `[lints] workspace = true`.
2. `/home/console-admin/Github/Label-Agent/crates/label-inventory/src/lib.rs`:
   Pure manifest:
   ```rust
   //! Snipe-IT REST API client and 6-level location resolver for Label-Agent.

   pub mod client;
   pub mod error;
   pub mod models;

   pub use crate::client::{InventoryConfig, InventoryRepository, SnipeItClient};
   pub use crate::error::InventoryError;
   pub use crate::models::{AssetDto, LocationDto, PaginatedResponse};
   ```
3. `/home/console-admin/Github/Label-Agent/crates/label-inventory/src/error.rs`:
   Define `#[non_exhaustive] #[derive(Debug, Error, Clone, PartialEq, Eq)] pub enum InventoryError`:
   - `EmptyBaseUrl`
   - `EmptyApiToken`
   - `Unauthorized`
   - `RateLimited`
   - `NotFound { resource: String }`
   - `HttpTransport { message: String }`
   - `Deserialization { message: String }`
   - `Core(#[from] label_core::CoreError)`
   Include unit tests in `#[cfg(test)] mod tests`.
4. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo006_inventory_manifest.md`.

---

## `WO-007`: Inventory Client & Models (`crates/label-inventory` Part B)
**Assigned Files (Create ONLY these 2 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-inventory/src/models.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct LocationDto` (private fields `id: u64`, `name: String`, `parent_id: Option<u64>`, constructor `#[must_use] pub fn new(id: u64, name: String, parent_id: Option<u64>) -> Self`, getters `id(&self) -> u64`, `name(&self) -> &str`, `parent_id(&self) -> Option<u64>`).
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct AssetDto` (private fields `id: u64`, `asset_tag: String`, `name: String`, `serial: Option<String>`, `location_id: Option<u64>`, constructor `#[must_use] pub fn new(...) -> Self`, getters `id(&self) -> u64`, `asset_tag(&self) -> &str`, `name(&self) -> &str`, `serial(&self) -> Option<&str>`, `location_id(&self) -> Option<u64>`).
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct PaginatedResponse<T>` (private fields `total: usize`, `rows: Vec<T>`, constructor `#[must_use] pub fn new(total: usize, rows: Vec<T>) -> Self`, getters `total(&self) -> usize`, `rows(&self) -> &[T]`).
   - Unit tests for JSON deserialization in `#[cfg(test)] mod tests`.
2. `/home/console-admin/Github/Label-Agent/crates/label-inventory/src/client.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)] pub struct InventoryConfig` (`base_url: String`, `api_token: String`, `timeout: Duration`), `pub fn new(base_url: &str, api_token: &str, timeout: Duration) -> Result<Self, InventoryError>` (rejects empty trimmed `base_url` or `api_token`), getters `base_url(&self) -> &str`, `api_token(&self) -> &str`, `timeout(&self) -> Duration`, `#[must_use] pub fn bearer_header(&self) -> String` (`format!("Bearer {}", self.api_token)`), `#[must_use] pub fn endpoint_url(&self, path: &str) -> String`.
   - `pub trait InventoryRepository`:
     - `fn find_location_by_code(&self, code: &str) -> Result<LocationDto, InventoryError>;`
     - `fn resolve_container_lineage(&self, container_code: &str) -> Result<LocationPath, InventoryError>;`
     - `fn list_assets_in_location(&self, location_id: u64) -> Result<Vec<AssetDto>, InventoryError>;`
     - `fn checkout_asset_to_location(&self, asset_tag: &str, location_id: u64) -> Result<(), InventoryError>;`
     - `fn relocate_container(&self, container_id: u64, new_parent_id: u64) -> Result<(), InventoryError>;`
   - `#[non_exhaustive] #[derive(Debug, Clone)] pub struct SnipeItClient` holding `config: InventoryConfig` and `agent: ureq::Agent` (`ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(5)).timeout_read(config.timeout()).build()`), with `#[must_use] pub fn new(config: InventoryConfig) -> Self`, `#[must_use] pub fn config(&self) -> &InventoryConfig`, `#[must_use] pub fn map_http_status(status: u16, resource: &str) -> InventoryError` (`401 | 403 => InventoryError::Unauthorized`, `404 => InventoryError::NotFound { resource: resource.to_owned() }`, `429 => InventoryError::RateLimited`, `code => InventoryError::HttpTransport { message: format!("HTTP status {code} for {resource}") }`), and `impl InventoryRepository for SnipeItClient`.
   - Unit tests verifying `InventoryConfig::new`, `bearer_header()`, `endpoint_url()`, and `map_http_status(401/404/429/500)`.
3. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo007_inventory_client.md`.

---

## `WO-008`: Render Manifest & Error (`crates/label-render` Part A)
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-render/Cargo.toml`:
   Workspace metadata, `[dependencies]` (`label-core.workspace = true`, `qrcode.workspace = true`, `thiserror.workspace = true`), `[lints] workspace = true`.
2. `/home/console-admin/Github/Label-Agent/crates/label-render/src/lib.rs`:
   Pure manifest:
   ```rust
   //! Pure-Rust QR code, 1-bit BMP, ODT, Brother LBX, and Dymo 30256 PDF
   //! label compiler and RAM (`tmpfs`) spooler for Label-Agent.

   pub mod bmp;
   pub mod dymo_pdf;
   pub mod error;
   pub mod lbx;
   pub mod odt;
   pub mod qr;
   pub mod spool;
   pub mod zip_store;

   pub use crate::bmp::encode_monochrome_bmp_192;
   pub use crate::dymo_pdf::compile_dymo_pdf;
   pub use crate::error::RenderError;
   pub use crate::lbx::compile_lbx_label;
   pub use crate::odt::{compile_odt_label, escape_xml};
   pub use crate::qr::QrMatrix;
   pub use crate::spool::SpoolManager;
   pub use crate::zip_store::ZipStoreBuilder;
   ```
3. `/home/console-admin/Github/Label-Agent/crates/label-render/src/error.rs`:
   Define `#[non_exhaustive] #[derive(Debug, Error, Clone, PartialEq, Eq)] pub enum RenderError`:
   - `EmptyPayload`
   - `QrEncoding { message: String }`
   - `InvalidSpoolIdentifier`
   - `Io { message: String }`
   Include unit tests in `#[cfg(test)] mod tests`.
4. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo008_render_manifest.md`.

---

## `WO-009`: Render QR, BMP & Spool (`crates/label-render` Part B)
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-render/src/qr.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)] pub struct QrMatrix` with private `width: usize` and `modules: Vec<bool>`.
   - `pub fn encode(payload: &str) -> Result<Self, RenderError>` using `qrcode::QrCode::with_error_correction_level(payload.as_bytes(), qrcode::EcLevel::M)` (returns `RenderError::EmptyPayload` if `payload.trim().is_empty()`).
   - `#[must_use] pub const fn width(&self) -> usize`, `#[must_use] pub fn is_dark(&self, x: usize, y: usize) -> bool`, `#[must_use] pub fn to_svg_string(&self, size_px: u32) -> String`.
   - Unit tests in `#[cfg(test)] mod tests`.
2. `/home/console-admin/Github/Label-Agent/crates/label-render/src/bmp.rs`:
   - `pub fn encode_monochrome_bmp_192(qr_payload: &str) -> Result<Vec<u8>, RenderError>`:
     Encodes `qr_payload` via `QrMatrix::encode(qr_payload)?` and renders a `192 x 192` 1-bit uncompressed Windows `BITMAPINFOHEADER` BMP:
     - 14-byte `BITMAPFILEHEADER` (`b"BM"`, file size `4670` little-endian `u32`, reserved `0`, pixel offset `62` little-endian `u32`)
     - 40-byte `BITMAPINFOHEADER` (size `40`, width `192`, height `192`, planes `1`, bit_count `1`, compression `0`, image_size `4608`, x/y ppm `2835`, clr_used `2`, clr_important `2`)
     - 8-byte color table (`[0, 0, 0, 0, 255, 255, 255, 0]`: index 0 = black, index 1 = white)
     - `4608` bytes of pixel data (`192` rows $\times$ `24` bytes/row), mapping scaled `QrMatrix` modules to bit `0` (black) and quiet zone/light modules to bit `1` (white).
   - Unit tests asserting `bytes.len() == 4670`, `&bytes[0..2] == b"BM"`, and pixel offset `62`.
3. `/home/console-admin/Github/Label-Agent/crates/label-render/src/spool.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)] pub struct SpoolManager` with private `spool_dir: PathBuf`.
   - `#[must_use] pub fn new(spool_dir: PathBuf) -> Self` and `impl Default for SpoolManager` defaulting to `PathBuf::from("/tmp/label-agent")`.
   - `#[must_use] pub fn spool_dir(&self) -> &Path`.
   - `pub fn sanitize_stem(label_id: &str) -> Result<String, RenderError>`: strips path components/separators (`/`, `\`, `..`) by replacing any char that is not ASCII alphanumeric or `-` with `_`, collapsing consecutive `_` into a single `_`, trimming leading/trailing `_`, and returning `RenderError::InvalidSpoolIdentifier` if the resulting string is empty or consists only of `-`/`_`.
   - `pub fn write_artifact(&self, label_id: &str, extension: &str, bytes: &[u8]) -> Result<PathBuf, RenderError>`: creates `self.spool_dir` if absent, sanitizes both `label_id` and `extension`, writes `<stem>.<ext>` inside `self.spool_dir`, and returns the `PathBuf`.
   - Unit tests verifying `"BOX-001/../../etc"` sanitizes to `"BOX-001_etc"` and writes inside a temp dir.
4. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo009_render_qr_bmp.md`.

---

## `WO-010a`: Render ZIP & ODT (`crates/label-render` Part C)
**Assigned Files (Create ONLY these 2 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-render/src/zip_store.rs`:
   - Pure-Rust uncompressed (`STORE`, compression method `0`) PKZIP archive builder `#[non_exhaustive] #[derive(Debug, Clone, Default)] pub struct ZipStoreBuilder`:
     - `#[must_use] pub fn new() -> Self`
     - `pub fn add_file(&mut self, name: &str, data: &[u8])`
     - `#[must_use] pub fn finish(self) -> Vec<u8>` (writes Local File Headers `0x04034b50`, raw data, Central Directory Headers `0x02014b50`, and End of Central Directory `0x06054b50` with IEEE 802.3 CRC-32 `crc32(data: &[u8]) -> u32`).
   - Unit tests verifying `PK\x03\x04` magic and entry ordering.
2. `/home/console-admin/Github/Label-Agent/crates/label-render/src/odt.rs`:
   - `#[must_use] pub fn escape_xml(input: &str) -> String`: replaces `&` with `&amp;`, `<` with `&lt;`, `>` with `&gt;`, `"` with `&quot;`, and `'` with `&apos;`.
   - `pub fn compile_odt_label(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError>`:
     Builds a `BoxLabels`-compatible `.odt` archive using `ZipStoreBuilder`:
     - Entry 1 (`mimetype`): exact bytes `b"application/vnd.oasis.opendocument.text"` (uncompressed first entry per ODF 1.2 spec).
     - Entry 2 (`styles.xml`): ODF styles with `fo:page-width="4in"`, `fo:page-height="2.3098in"`, `fo:margin-top="0.1701in"`, `fo:margin-bottom="0.1701in"`, `fo:margin-left="0.1701in"`, `fo:margin-right="0.1701in"`.
     - Entry 3 (`content.xml`): XML-escaped `spec.label_id()`, `spec.location_path().breadcrumb()`, `spec.category_icon()`, `0.7in` QR frame (`Pictures/qr.svg`), and bulleted items prefixed with `\u{2022}`.
     - Entry 4 (`Pictures/qr.svg`): SVG from `QrMatrix::encode(&spec.qr_payload_url())?.to_svg_string(210)`.
     - Entry 5 (`META-INF/manifest.xml`): ODF manifest registering `/`, `styles.xml`, `content.xml`, and `Pictures/qr.svg`.
   - Unit tests verifying `mimetype` is at byte offset `30..69` (`b"mimetypeapplication/vnd.oasis.opendocument.text"`) and XML special chars `"AT&T <Cat6> & 'Fiber'"` are escaped.
3. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo010a_render_odt.md`.

---

## `WO-010b`: Render LBX & Dymo PDF (`crates/label-render` Part D)
**Assigned Files (Create ONLY these 2 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-render/src/lbx.rs`:
   - `pub fn compile_lbx_label(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError>`:
     Uses `ZipStoreBuilder`, `escape_xml`, and `encode_monochrome_bmp_192(&spec.qr_payload_url())?` to produce a Brother P-Touch `.lbx` archive containing:
     - `prop.xml` (Brother P-Touch metadata)
     - `label.xml` (XML-escaped `spec.label_id()`, `spec.location_path().breadcrumb()`, and reference to `Object0.bmp`)
     - `Object0.bmp` (`4670`-byte `192x192` 1-bit monochrome BMP)
   - Unit tests verifying `PK\x03\x04` header, presence of `Object0.bmp` and `label.xml`, and XML escaping.
2. `/home/console-admin/Github/Label-Agent/crates/label-render/src/dymo_pdf.rs`:
   - `pub fn compile_dymo_pdf(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError>`:
     Generates a valid pure-Rust PDF-1.4 document (`%PDF-1.4` ... `%%EOF`) with page `/MediaBox [0 0 288.00 166.31]` (`4.0" x 2.3098"` Dymo 30256 geometry), rendering the QR code via `QrMatrix::encode(&spec.qr_payload_url())?`, the header `spec.label_id()`, breadcrumb `spec.location_path().breadcrumb()`, and bulleted items (`• <item>`).
   - Unit tests verifying `%PDF-1.4` header, `/MediaBox [0 0 288.00 166.31]`, and `%%EOF` trailer.
3. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo010b_render_lbx_pdf.md`.

---

## `WO-011a`: Print Manifest & Error (`crates/label-print` Part A)
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-print/Cargo.toml`:
   Workspace metadata, `[dependencies]` (`label-core.workspace = true`, `thiserror.workspace = true`), `[lints] workspace = true`.
2. `/home/console-admin/Github/Label-Agent/crates/label-print/src/lib.rs`:
   Pure manifest:
   ```rust
   //! Hardware print transport abstractions and CUPS (`lpr`) dispatcher for Label-Agent.

   pub mod cups;
   pub mod error;
   pub mod transport;

   pub use crate::cups::CupsTransport;
   pub use crate::error::PrintError;
   pub use crate::transport::{DryRunTransport, PrintJob, PrintReceipt, PrintTransport};
   ```
3. `/home/console-admin/Github/Label-Agent/crates/label-print/src/error.rs`:
   Define `#[non_exhaustive] #[derive(Debug, Error, Clone, PartialEq, Eq)] pub enum PrintError`:
   - `EmptyQueueName`
   - `ZeroCopies`
   - `MissingSpoolFile { path: String }`
   - `SpoolerBinaryNotFound { path: String }`
   - `CommandFailed { status: i32, stderr: String }`
   - `Io { message: String }`
   Include unit tests in `#[cfg(test)] mod tests`.
4. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo011a_print_manifest.md`.

---

## `WO-011b`: Print Transport & CUPS (`crates/label-print` Part B)
**Assigned Files (Create ONLY these 2 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-print/src/transport.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)] pub struct PrintJob` (`queue_name: String`, `artifact_path: PathBuf`, `format: LabelFormat`, `copies: u8`), `pub fn new(queue_name: &str, artifact_path: PathBuf, format: LabelFormat, copies: u8) -> Result<Self, PrintError>` (rejects empty `queue_name` or `copies == 0`), getters `queue_name(&self) -> &str`, `artifact_path(&self) -> &Path`, `format(&self) -> LabelFormat`, `copies(&self) -> u8`.
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)] pub struct PrintReceipt` (`job_id: String`, `queue_name: String`, `mode: PrintMode`), constructor `#[must_use] pub fn new(...) -> Self` and getters `job_id(&self) -> &str`, `queue_name(&self) -> &str`, `mode(&self) -> PrintMode`.
   - `pub trait PrintTransport`: `fn dispatch(&self, job: &PrintJob) -> Result<PrintReceipt, PrintError>;`
   - `#[non_exhaustive] #[derive(Debug, Default)] pub struct DryRunTransport` with private `jobs: Mutex<Vec<PrintJob>>`, `#[must_use] pub fn new() -> Self`, `pub fn with_jobs<R>(&self, f: impl FnOnce(&[PrintJob]) -> R) -> R` (using `.expect("dry-run jobs mutex is never poisoned")` — NO `.into_inner()` per `GRS-007`), and `impl PrintTransport for DryRunTransport`.
   - Unit tests in `#[cfg(test)] mod tests`.
2. `/home/console-admin/Github/Label-Agent/crates/label-print/src/cups.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)] pub struct CupsTransport` (`lpr_binary: PathBuf`, `default_queue: String`), `#[must_use] pub fn new(lpr_binary: PathBuf, default_queue: String) -> Self`, `impl Default for CupsTransport` (`/usr/bin/lpr` and `"Dymo_LabelWriter_450"`), getters `lpr_binary(&self) -> &Path`, `default_queue(&self) -> &str`, `#[must_use] pub fn build_lpr_args(&self, job: &PrintJob) -> Vec<String>` (`vec!["-P".to_owned(), job.queue_name().to_owned(), "-#".to_owned(), job.copies().to_string(), job.artifact_path().display().to_string()]`), and `impl PrintTransport for CupsTransport` (checks `job.artifact_path().exists()`, checks `self.lpr_binary.exists()`, invokes `Command::new(&self.lpr_binary)`).
   - Unit tests verifying `build_lpr_args` and missing file error handling.
3. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo011b_print_transport.md`.

---

## `WO-012a`: Voice Manifest & Error (`crates/label-voice` Part A)
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-voice/Cargo.toml`:
   Workspace metadata, `[dependencies]` (`label-core.workspace = true`, `thiserror.workspace = true`), `[lints] workspace = true`.
2. `/home/console-admin/Github/Label-Agent/crates/label-voice/src/lib.rs`:
   Pure manifest:
   ```rust
   //! Voice command capture abstractions and natural-language intent parser
   //! for Label-Agent.

   pub mod capture;
   pub mod error;
   pub mod parser;

   pub use crate::capture::{PcmConfig, StaticUtteranceQueue, UtteranceSource};
   pub use crate::error::VoiceError;
   pub use crate::parser::parse_spoken_command;
   ```
3. `/home/console-admin/Github/Label-Agent/crates/label-voice/src/error.rs`:
   Define `#[non_exhaustive] #[derive(Debug, Error, Clone, PartialEq, Eq)] pub enum VoiceError`:
   - `EmptyUtterance`
   - `UnrecognizedIntent { utterance: String }`
   - `MissingIdentifier { context: &'static str }`
   - `AudioCapture { message: String }`
   Include unit tests in `#[cfg(test)] mod tests`.
4. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo012a_voice_manifest.md`.

---

## `WO-012b`: Voice Capture & Parser (`crates/label-voice` Part B)
**Assigned Files (Create ONLY these 2 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-voice/src/capture.rs`:
   - `#[non_exhaustive] #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct PcmConfig` (`sample_rate_hz: u32`, `channels: u8`, `frame_ms: u16`), `#[must_use] pub const fn pi3b_alsa_default() -> Self` (`16_000`, `1`, `20`), `impl Default for PcmConfig`, getters `sample_rate_hz`, `channels`, `frame_ms`, `#[must_use] pub const fn samples_per_frame(&self) -> usize`.
   - `pub trait UtteranceSource`: `fn next_utterance(&mut self) -> Result<Option<String>, VoiceError>;`
   - `#[non_exhaustive] #[derive(Debug, Clone, Default)] pub struct StaticUtteranceQueue` holding `VecDeque<String>`, `#[must_use] pub fn new(utterances: Vec<String>) -> Self`, `#[must_use] pub fn len(&self) -> usize`, `#[must_use] pub fn is_empty(&self) -> bool` (required alongside `len()` by `clippy::len_without_is_empty`!), and `impl UtteranceSource for StaticUtteranceQueue`.
   - Unit tests in `#[cfg(test)] mod tests`.
2. `/home/console-admin/Github/Label-Agent/crates/label-voice/src/parser.rs`:
   - `pub fn parse_spoken_command(utterance: &str) -> Result<VoiceCommandIntent, VoiceError>`:
     Deterministic natural-language command parser supporting:
     - Print box label commands (e.g. `"print box label for BOX-042"`, `"print label for BOX-042 on brother"`, `"print odt label for BOX-042"`): extracts box identifier token (`BOX-...` or token after `"box"`/`"for"`) and maps format keywords (`"brother"`/`"ptouch"`/`"lbx"` -> `LabelFormat::BrotherPtouchLbx`, `"odt"`/`"boxlabels"` -> `LabelFormat::BoxLabelsOdt`, default -> `LabelFormat::Dymo30256Pdf`).
     - Query box contents commands (e.g. `"what is inside box BOX-042"`, `"list contents of box BOX-042"`, `"query box BOX-042"`): returns `VoiceCommandIntent::QueryBoxContents { box_code }`.
     - Check out asset commands (e.g. `"check out asset ASSET-109 to box BOX-042"`, `"assign ASSET-109 to BOX-042"`): returns `VoiceCommandIntent::CheckOutAssetToBox { asset_tag, box_code }`.
     - Relocate box commands (e.g. `"move box BOX-042 to S2"`, `"relocate box BOX-042 to RACK-1"`): returns `VoiceCommandIntent::RelocateBox { box_code, target_parent_code }`.
   - Unit tests covering all 4 intents, all 3 label formats, empty input, and unrecognized utterance.
3. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo012b_voice_parser.md`.

---

## `WO-013a`: Daemon Manifest & CLI (`crates/label-agent-daemon` Part A)
**Assigned Files (Create ONLY these 3 files)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-agent-daemon/Cargo.toml`:
   `name = "label-agent-daemon"`, `[[bin]] name = "label-agent" path = "src/main.rs"`, workspace metadata, `[dependencies]` (`label-core.workspace = true`, `label-inventory.workspace = true`, `label-render.workspace = true`, `label-print.workspace = true`, `label-voice.workspace = true`, `lexopt.workspace = true`, `thiserror.workspace = true`), `[lints] workspace = true`.
2. `/home/console-admin/Github/Label-Agent/crates/label-agent-daemon/src/main.rs`:
   ```rust
   //! Binary entrypoint for the `label-agent` autonomous appliance daemon.

   mod app;
   mod cli;

   use std::env;
   use std::process::ExitCode;

   fn main() -> ExitCode {
       let args: Vec<String> = env::args().skip(1).collect();
       match app::run_from_args(&args) {
           Ok(summary) => {
               println!("{summary}");
               ExitCode::SUCCESS
           }
           Err(err) => {
               eprintln!("label-agent error: {err}");
               ExitCode::FAILURE
           }
       }
   }
   ```
3. `/home/console-admin/Github/Label-Agent/crates/label-agent-daemon/src/cli.rs`:
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct CliOptions` (`command: Option<String>`, `print_mode: PrintMode`, `spool_dir: PathBuf`, `printer_queue: String`, `inventory_base_url: String`, `daemon_mode: bool`, `show_help: bool`).
   - `pub fn parse_cli_args(args: &[String]) -> Result<CliOptions, String>` using `lexopt::Parser::from_args(args)` supporting `--command` / `-c`, `--dry-run`, `--spool-dir`, `--printer` / `-p`, `--inventory-url`, `--daemon`, `--help` / `-h`.
   - `#[must_use] pub fn help_banner() -> &'static str`.
   - Unit tests in `#[cfg(test)] mod tests`.
4. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo013a_daemon_cli.md`.

---

## `WO-013b`: Daemon Orchestrator App (`crates/label-agent-daemon` Part B)
**Assigned Files (Create ONLY this 1 file)**:
1. `/home/console-admin/Github/Label-Agent/crates/label-agent-daemon/src/app.rs`:
   - Implements `pub fn run_from_args(args: &[String]) -> Result<String, String>` and `pub fn execute_spoken_command<R: InventoryRepository, T: PrintTransport>(utterance: &str, repo: &R, spooler: &SpoolManager, transport: &T, queue_name: &str, base_url: &str) -> Result<String, String>`.
   - Provides a deterministic `DemoInventoryRepository` (used in `--dry-run` or when `SNIPEIT_API_KEY` is not set) pre-populated with a valid 6-level lineage (`Main Campus > F1 > R101 > Rack1 > S1 > BOX-042`) and sample assets (`Cat6 Patch Cable`, `Fluke Network Tester`, `SFP+ Transceiver`) so `--command "print box label for BOX-042" --dry-run` works end-to-end out of the box!
   - Integration tests in `#[cfg(test)] mod tests` testing `run_from_args` with `--help`, `--command "print box label for BOX-042" --dry-run`, `--command "print label for BOX-042 on brother" --dry-run`, `--command "print odt label for BOX-042" --dry-run`, and `--command "what is inside box BOX-042" --dry-run`.
2. Summary path: `/home/console-admin/Github/Label-Agent/.agents/knowledge/subagent_summaries/20261107_wo013b_daemon_app.md`.
