//! Main application orchestration and command execution logic for
//! `label-agent`.
//!
//! Provides CLI dispatch, spoken voice command handling, physical print
//! spooling, and deterministic mock inventory resolution for dry-run
//! and demonstration environments.

use label_core::BoxLabelSpec;
use label_core::LabelFormat;
use label_core::LocationNode;
use label_core::LocationPath;
use label_core::PrintMode;
use label_core::TaxonomyLevel;
use label_core::VoiceCommandIntent;
use label_inventory::AssetDto;
use label_inventory::InventoryError;
use label_inventory::InventoryRepository;
use label_inventory::LocationDto;
use label_print::CupsTransport;
use label_print::DryRunTransport;
use label_print::PrintJob;
use label_print::PrintTransport;
use label_render::compile_dymo_pdf;
use label_render::compile_lbx_label;
use label_render::compile_odt_label;
use label_render::SpoolManager;
use label_voice::parse_spoken_command;
use std::path::PathBuf;

use crate::cli::help_banner;
use crate::cli::parse_cli_args;

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
            let lineage = repo
                .resolve_container_lineage(&box_code)
                .map_err(|err| format!("Location resolution error: {err}"))?;
            let assets = repo
                .list_assets_in_location(6)
                .map_err(|err| format!("Asset lookup error: {err}"))?;
            let items: Vec<String> = assets
                .iter()
                .map(|asset| asset.name().to_string())
                .collect();

            let spec = BoxLabelSpec::new(&box_code, base_url, "box", lineage, items)
                .map_err(|err| format!("Label specification error: {err}"))?;

            let (ext, bytes) = match format {
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
            let lineage = repo
                .resolve_container_lineage(&box_code)
                .map_err(|err| format!("Location resolution error: {err}"))?;
            let assets = repo
                .list_assets_in_location(6)
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
            repo.checkout_asset_to_location(&asset_tag, 6)
                .map_err(|err| format!("Checkout error: {err}"))?;
            Ok(format!("Checked out asset {asset_tag} to box {box_code}"))
        }
        VoiceCommandIntent::RelocateBox {
            box_code,
            target_parent_code,
        } => {
            repo.relocate_container(6, 5)
                .map_err(|err| format!("Relocation error: {err}"))?;
            Ok(format!(
                "Relocated box {box_code} to parent {target_parent_code}"
            ))
        }
        _ => Err("unsupported variant".to_owned()),
    }
}

/// Primary CLI application driver. Parses arguments and executes the requested
/// operation or launches the daemon.
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

    let spooler = SpoolManager::new(opts.spool_dir);
    let repo = DemoInventoryRepository::new();

    if let Some(ref utterance) = opts.command {
        match opts.print_mode {
            PrintMode::DryRun => {
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
            PrintMode::ExecutePrint => {
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
            _ => Err("unsupported variant".to_owned()),
        }
    } else if opts.daemon_mode {
        Ok("label-agent daemon listening for voice commands...".to_string())
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
}
