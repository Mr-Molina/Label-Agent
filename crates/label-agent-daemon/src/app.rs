//! Main application orchestration and command execution logic for
//! `label-agent`.
//!
//! Provides CLI dispatch, spoken voice command handling, `59 mm x 102 mm`
//! BT320 TSPL2 / PDF / ODT / LBX rendering, physical `/dev/bt320-printer`
//! (`/dev/usb/lp0`) and CUPS spooling, and an HTTP health/control listener.

use std::io::Read as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;

use label_core::BoxLabelSpec;
use label_core::LabelFormat;
use label_core::LocationNode;
use label_core::LocationPath;
use label_core::PrintMode;
use label_core::TaxonomyLevel;
use label_core::VoiceCommandIntent;
use label_inventory::AssetDto;
use label_inventory::InventoryConfig;
use label_inventory::InventoryError;
use label_inventory::InventoryRepository;
use label_inventory::LocationDto;
use label_inventory::SnipeItClient;
use label_print::CupsTransport;
use label_print::DryRunTransport;
use label_print::PrintJob;
use label_print::PrintTransport;
use label_print::UsbLpTransport;
use label_print::FALLBACK_USB_LP_DEVICE;
use label_render::compile_bt320_tspl;
use label_render::compile_dymo_pdf;
use label_render::compile_lbx_label;
use label_render::compile_odt_label;
use label_render::SpoolManager;
use label_voice::parse_spoken_command;

use crate::cli::help_banner;
use crate::cli::parse_cli_args;
use crate::cli::CliOptions;

/// Deterministic mock repository implementing [`InventoryRepository`]
/// with sample 6-level lineage and assets for dry-run and demo execution.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub(crate) struct DemoInventoryRepository;

impl DemoInventoryRepository {
    /// Creates a new demo repository instance.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self
    }
}

impl InventoryRepository for DemoInventoryRepository {
    fn find_location_by_code(&self, code: &str) -> Result<LocationDto, InventoryError> {
        Ok(LocationDto::new(6, code.to_string(), Some(5)))
    }

    fn resolve_container_lineage(
        &self,
        container_code: &str,
    ) -> Result<LocationPath, InventoryError> {
        let nodes = vec![
            LocationNode::new(1, "Main Campus", None, TaxonomyLevel::Building),
            LocationNode::new(2, "F1", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "R101", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "Rack1", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "S1", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, container_code, Some(5), TaxonomyLevel::ContainerBox),
        ];
        LocationPath::from_container_lineage(nodes).map_err(InventoryError::from)
    }

    fn list_assets_in_location(&self, _location_id: u64) -> Result<Vec<AssetDto>, InventoryError> {
        Ok(vec![
            AssetDto::new(
                101,
                "AST-001".to_string(),
                "Cat6 Patch Cable".to_string(),
                None,
                Some(6),
            ),
            AssetDto::new(
                102,
                "AST-002".to_string(),
                "Fluke Network Tester".to_string(),
                None,
                Some(6),
            ),
            AssetDto::new(
                103,
                "AST-003".to_string(),
                "SFP+ Transceiver".to_string(),
                None,
                Some(6),
            ),
        ])
    }

    fn checkout_asset_to_location(
        &self,
        _asset_tag: &str,
        _location_id: u64,
    ) -> Result<(), InventoryError> {
        Ok(())
    }

    fn relocate_container(
        &self,
        _container_id: u64,
        _new_parent_id: u64,
    ) -> Result<(), InventoryError> {
        Ok(())
    }
}

/// Resolves the active USB character device path (`/dev/bt320-printer` or
/// fallback `/dev/usb/lp0`).
#[must_use]
fn resolve_usb_device_path(configured: &Path) -> PathBuf {
    if configured.exists() {
        configured.to_path_buf()
    } else {
        let fallback = PathBuf::from(FALLBACK_USB_LP_DEVICE);
        if fallback.exists() {
            fallback
        } else {
            configured.to_path_buf()
        }
    }
}

/// Executes a natural-language or voice command string against the inventory
/// repository and print pipeline.
///
/// # Errors
///
/// Returns an error message string if parsing, inventory resolution,
/// rendering, or print dispatch fails.
pub(crate) fn execute_spoken_command<R: InventoryRepository, T: PrintTransport>(
    utterance: &str,
    repo: &R,
    spooler: &SpoolManager,
    transport: &T,
    queue_name: &str,
    base_url: &str,
) -> Result<String, String> {
    let intent =
        parse_spoken_command(utterance).map_err(|err| format!("Voice parsing failed: {err}"))?;

    match intent {
        VoiceCommandIntent::PrintBoxLabel { box_code, format } => {
            let loc = repo
                .find_location_by_code(&box_code)
                .map_err(|err| format!("Location lookup error: {err}"))?;
            let lineage = repo
                .resolve_container_lineage(&box_code)
                .map_err(|err| format!("Location resolution error: {err}"))?;
            let assets = repo
                .list_assets_in_location(loc.id())
                .map_err(|err| format!("Asset lookup error: {err}"))?;
            let items: Vec<String> = assets
                .iter()
                .map(|asset| asset.name().to_string())
                .collect();

            let spec = BoxLabelSpec::new(&box_code, base_url, "box", lineage, items)
                .map_err(|err| format!("Label specification error: {err}"))?;

            let (ext, bytes) = match format {
                LabelFormat::Bt320Tspl => (
                    "tspl",
                    compile_bt320_tspl(&spec)
                        .map_err(|err| format!("TSPL2 rendering failed: {err}"))?,
                ),
                LabelFormat::Dymo30256Pdf => (
                    "pdf",
                    compile_dymo_pdf(&spec)
                        .map_err(|err| format!("PDF rendering failed: {err}"))?,
                ),
                LabelFormat::BoxLabelsOdt => (
                    "odt",
                    compile_odt_label(&spec)
                        .map_err(|err| format!("ODT rendering failed: {err}"))?,
                ),
                LabelFormat::BrotherPtouchLbx => (
                    "lbx",
                    compile_lbx_label(&spec)
                        .map_err(|err| format!("LBX rendering failed: {err}"))?,
                ),
                _ => return Err("unsupported variant".to_owned()),
            };

            let artifact_path = spooler
                .write_artifact(&box_code, ext, &bytes)
                .map_err(|err| format!("Spool artifact error: {err}"))?;

            let job = PrintJob::new(queue_name, artifact_path, format, 1)
                .map_err(|err| format!("Print job creation error: {err}"))?;

            let receipt = transport
                .dispatch(&job)
                .map_err(|err| format!("Print dispatch error: {err}"))?;

            Ok(format!(
                "Printed box label for {box_code} ({ext}) - job ID: {}",
                receipt.job_id()
            ))
        }
        VoiceCommandIntent::QueryBoxContents { box_code } => {
            let loc = repo
                .find_location_by_code(&box_code)
                .map_err(|err| format!("Location lookup error: {err}"))?;
            let lineage = repo
                .resolve_container_lineage(&box_code)
                .map_err(|err| format!("Location resolution error: {err}"))?;
            let assets = repo
                .list_assets_in_location(loc.id())
                .map_err(|err| format!("Asset lookup error: {err}"))?;
            let mut lines = Vec::new();
            lines.push(format!(
                "Box {box_code} located at {}",
                lineage.breadcrumb()
            ));
            lines.push("Contents:".to_string());
            for asset in &assets {
                lines.push(format!(" - {} ({})", asset.name(), asset.asset_tag()));
            }
            Ok(lines.join("\n"))
        }
        VoiceCommandIntent::CheckOutAssetToBox {
            asset_tag,
            box_code,
        } => {
            let loc = repo
                .find_location_by_code(&box_code)
                .map_err(|err| format!("Location lookup error: {err}"))?;
            repo.checkout_asset_to_location(&asset_tag, loc.id())
                .map_err(|err| format!("Checkout error: {err}"))?;
            Ok(format!("Checked out asset {asset_tag} to box {box_code}"))
        }
        VoiceCommandIntent::RelocateBox {
            box_code,
            target_parent_code,
        } => {
            let box_loc = repo
                .find_location_by_code(&box_code)
                .map_err(|err| format!("Box lookup error: {err}"))?;
            let parent_loc = repo
                .find_location_by_code(&target_parent_code)
                .map_err(|err| format!("Target parent lookup error: {err}"))?;
            repo.relocate_container(box_loc.id(), parent_loc.id())
                .map_err(|err| format!("Relocation error: {err}"))?;
            Ok(format!(
                "Relocated box {box_code} to parent {target_parent_code}"
            ))
        }
        _ => Err("unsupported variant".to_owned()),
    }
}

/// Dispatches a single utterance using configured CLI options and environment.
fn dispatch_utterance_with_opts(opts: &CliOptions, utterance: &str) -> Result<String, String> {
    let spooler = SpoolManager::new(opts.spool_dir.clone());
    let usb_path = resolve_usb_device_path(&opts.usb_device);
    let api_key = std::env::var("SNIPEIT_API_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty());

    match (opts.print_mode, api_key) {
        (PrintMode::DryRun, _) => {
            let repo = DemoInventoryRepository::new();
            let transport = DryRunTransport::new();
            execute_spoken_command(
                utterance,
                &repo,
                &spooler,
                &transport,
                &opts.printer_queue,
                &opts.inventory_base_url,
            )
        }
        (PrintMode::ExecutePrint, Some(token)) => {
            let cfg = InventoryConfig::new(
                &opts.inventory_base_url,
                &token,
                std::time::Duration::from_secs(10),
            )
            .map_err(|e| format!("Inventory config error: {e}"))?;
            let repo = SnipeItClient::new(cfg);
            if usb_path.exists() {
                let transport = UsbLpTransport::new(usb_path);
                execute_spoken_command(
                    utterance,
                    &repo,
                    &spooler,
                    &transport,
                    &opts.printer_queue,
                    &opts.inventory_base_url,
                )
            } else {
                let transport =
                    CupsTransport::new(PathBuf::from("/usr/bin/lpr"), opts.printer_queue.clone());
                execute_spoken_command(
                    utterance,
                    &repo,
                    &spooler,
                    &transport,
                    &opts.printer_queue,
                    &opts.inventory_base_url,
                )
            }
        }
        (PrintMode::ExecutePrint, None) => {
            let repo = DemoInventoryRepository::new();
            if usb_path.exists() {
                let transport = UsbLpTransport::new(usb_path);
                execute_spoken_command(
                    utterance,
                    &repo,
                    &spooler,
                    &transport,
                    &opts.printer_queue,
                    &opts.inventory_base_url,
                )
            } else {
                let transport =
                    CupsTransport::new(PathBuf::from("/usr/bin/lpr"), opts.printer_queue.clone());
                execute_spoken_command(
                    utterance,
                    &repo,
                    &spooler,
                    &transport,
                    &opts.printer_queue,
                    &opts.inventory_base_url,
                )
            }
        }
        _ => Err("unsupported variant".to_owned()),
    }
}

/// Handles a single incoming HTTP connection on the daemon control socket.
pub(crate) fn handle_http_stream(mut stream: TcpStream, opts: &CliOptions) {
    let mut buf = [0_u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(0) | Err(_) => return,
        Ok(n) => n,
    };
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");

    if first_line.starts_with("GET /healthz") {
        let usb_path = resolve_usb_device_path(&opts.usb_device);
        let usb_present = usb_path.exists();
        let body = format!(
            "{{\"status\":\"ok\",\"service\":\"label-agent\",\
            \"board\":\"aml-s905x-cc\",\"inventory_url\":\"{}\",\
            \"label_geometry\":\"59x102mm\",\"printer_protocol\":\"TSPL2\",\
            \"usb_device\":\"{}\",\"usb_connected\":{}}}\n",
            opts.inventory_base_url,
            usb_path.display(),
            usb_present
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
            Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
    } else if first_line.starts_with("POST /api/v1/command") {
        let body_part = req.split_once("\r\n\r\n").map_or("", |(_, b)| b).trim();
        let mut call_opts = opts.clone();
        if body_part.contains("\"dry_run\":true") || body_part.contains("\"dry_run\": true") {
            call_opts.print_mode = PrintMode::DryRun;
        }
        let cmd_text = extract_command_field(body_part).unwrap_or_else(|| body_part.to_owned());
        match dispatch_utterance_with_opts(&call_opts, &cmd_text) {
            Ok(msg) => {
                let escaped = msg.replace('"', "\\\"").replace('\n', "\\n");
                let json = format!("{{\"status\":\"ok\",\"result\":\"{escaped}\"}}\n");
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                    Content-Length: {}\r\nConnection: close\r\n\r\n{json}",
                    json.len()
                );
                let _ = stream.write_all(resp.as_bytes());
            }
            Err(err) => {
                let escaped = err.replace('"', "\\\"").replace('\n', "\\n");
                let json = format!("{{\"status\":\"error\",\"error\":\"{escaped}\"}}\n");
                let resp = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\
                    Content-Length: {}\r\nConnection: close\r\n\r\n{json}",
                    json.len()
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        }
    } else {
        let body = "{\"status\":\"not_found\"}\n";
        let resp = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\
            Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
    }
}

/// Extracts `"command":"..."` from a simple JSON payload or returns `None`.
fn extract_command_field(input: &str) -> Option<String> {
    let key_idx = input.find("\"command\"")?;
    let after_key = &input[key_idx + 9..];
    let colon_idx = after_key.find(':')?;
    let after_colon = after_key[colon_idx + 1..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    let rest = &after_colon[1..];
    let end_quote = rest.find('"')?;
    Some(rest[..end_quote].to_owned())
}

/// Primary CLI application driver. Parses arguments and executes the requested
/// operation or launches the daemon HTTP listener.
///
/// # Errors
///
/// Returns an error message string if arguments are invalid or command
/// execution fails.
pub(crate) fn run_from_args(args: &[String]) -> Result<String, String> {
    let opts = parse_cli_args(args)?;
    if opts.show_help {
        return Ok(help_banner().to_string());
    }

    if let Some(ref utterance) = opts.command {
        dispatch_utterance_with_opts(&opts, utterance)
    } else if opts.daemon_mode {
        let _ = std::fs::create_dir_all(&opts.spool_dir);
        let listener = TcpListener::bind(&opts.listen_addr)
            .map_err(|e| format!("Failed to bind {}: {e}", opts.listen_addr))?;
        for stream in listener.incoming().flatten() {
            handle_http_stream(stream, &opts);
        }
        Ok("label-agent daemon stopped.".to_string())
    } else {
        Ok(help_banner().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_from_args_help() {
        let args = vec!["--help".to_string()];
        let output = run_from_args(&args).expect("help should succeed");
        assert!(output.contains("Autonomous appliance daemon"));
    }

    #[test]
    fn test_run_from_args_print_box_label_dry_run() {
        let args = vec![
            "--command".to_string(),
            "print box label for BOX-042".to_string(),
            "--dry-run".to_string(),
        ];
        let output = run_from_args(&args).expect("print box label should succeed");
        assert!(output.contains("Printed box label for BOX-042 (pdf)"));
    }

    #[test]
    fn test_run_from_args_print_bt320_tspl_dry_run() {
        let args = vec![
            "--command".to_string(),
            "print tspl label for BOX-042".to_string(),
            "--dry-run".to_string(),
        ];
        let output = run_from_args(&args).expect("bt320 tspl print should succeed");
        assert!(output.contains("Printed box label for BOX-042 (tspl)"));
    }

    #[test]
    fn test_run_from_args_print_brother_dry_run() {
        let args = vec![
            "--command".to_string(),
            "print label for BOX-042 on brother".to_string(),
            "--dry-run".to_string(),
        ];
        let output = run_from_args(&args).expect("brother print should succeed");
        assert!(output.contains("Printed box label for BOX-042 (lbx)"));
    }

    #[test]
    fn test_run_from_args_print_odt_dry_run() {
        let args = vec![
            "--command".to_string(),
            "print odt label for BOX-042".to_string(),
            "--dry-run".to_string(),
        ];
        let output = run_from_args(&args).expect("odt print should succeed");
        assert!(output.contains("Printed box label for BOX-042 (odt)"));
    }

    #[test]
    fn test_run_from_args_query_box_dry_run() {
        let args = vec![
            "--command".to_string(),
            "what is inside box BOX-042".to_string(),
            "--dry-run".to_string(),
        ];
        let output = run_from_args(&args).expect("query box should succeed");
        assert!(output.contains("BOX-042"));
        assert!(output.contains("Main Campus > F1 > R101 > Rack1 > S1 > BOX-042"));
        assert!(output.contains("Cat6 Patch Cable"));
    }

    #[test]
    fn test_extract_command_field() {
        let json = r#"{"command": "print tspl label for BOX-042", "dry_run": true}"#;
        assert_eq!(
            extract_command_field(json).as_deref(),
            Some("print tspl label for BOX-042")
        );
    }
}
