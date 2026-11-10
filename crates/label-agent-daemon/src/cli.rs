//! Command-line argument parsing and configuration options for `label-agent`.

use std::path::PathBuf;

use label_core::PrintMode;
use label_inventory::DEFAULT_INVENTORY_URL;
use label_print::DEFAULT_BT320_DEVICE;
use lexopt::Arg;
use lexopt::Parser;

/// Default local HTTP control/health listen address.
pub(crate) const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:8090";

/// Runtime options parsed from command-line arguments.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliOptions {
    /// Optional spoken or textual command to execute directly.
    pub(crate) command: Option<String>,
    /// Print execution mode (direct hardware/CUPS or dry-run preview).
    pub(crate) print_mode: PrintMode,
    /// Root spool directory for rendering temporary artifacts.
    pub(crate) spool_dir: PathBuf,
    /// Target CUPS destination printer queue name.
    pub(crate) printer_queue: String,
    /// Target USB character device path for BT320 TSPL2 printing.
    pub(crate) usb_device: PathBuf,
    /// Base URL for the Snipe-IT inventory REST API.
    pub(crate) inventory_base_url: String,
    /// Bind address (`IP:PORT`) for the daemon HTTP control server.
    pub(crate) listen_addr: String,
    /// Whether to run continuously in background daemon mode.
    pub(crate) daemon_mode: bool,
    /// Whether `--help` was requested on the CLI.
    pub(crate) show_help: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        let inventory_base_url = std::env::var("SNIPEIT_BASE_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_INVENTORY_URL.to_owned());
        let usb_device = std::env::var("LABEL_AGENT_USB_DEVICE")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map_or_else(|| PathBuf::from(DEFAULT_BT320_DEVICE), PathBuf::from);
        let listen_addr = std::env::var("LABEL_AGENT_LISTEN_ADDR")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_LISTEN_ADDR.to_owned());

        Self {
            command: None,
            print_mode: PrintMode::ExecutePrint,
            spool_dir: PathBuf::from("/tmp/label-agent-spool"),
            printer_queue: "BT320_Printer".to_owned(),
            usb_device,
            inventory_base_url,
            listen_addr,
            daemon_mode: false,
            show_help: false,
        }
    }
}

/// Static help banner describing supported CLI options.
#[must_use]
pub(crate) fn help_banner() -> &'static str {
    "Autonomous appliance daemon for label printing.\n\n\
    Usage: label-agent [OPTIONS]\n\n\
    Options:\n  \
      -c, --command <CMD>       Spoken or text command to execute\n  \
          --dry-run             Simulate without sending print job to hardware\n  \
          --spool-dir <DIR>     Directory for temporary render artifacts\n  \
      -p, --printer <QUEUE>     Destination CUPS printer queue name\n  \
      -d, --device <PATH>       USB character device (/dev/bt320-printer)\n  \
          --inventory-url <URL> Base URL for Snipe-IT inventory API\n  \
          --listen <IP:PORT>    HTTP control/health listen address\n  \
          --daemon              Run continuously in background daemon mode\n  \
      -h, --help                Display this help banner\n"
}

/// Parses CLI arguments into [`CliOptions`].
///
/// # Errors
/// Returns an error message if argument parsing fails or an option is invalid.
pub(crate) fn parse_cli_args(args: &[String]) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut parser = Parser::from_args(args);

    while let Some(arg) = parser.next().map_err(|e| e.to_string())? {
        match arg {
            Arg::Short('c') | Arg::Long("command") => {
                let val = parser.value().map_err(|e| e.to_string())?;
                let text = val
                    .into_string()
                    .map_err(|os| format!("invalid UTF-8 argument: {}", os.to_string_lossy()))?;
                options.command = Some(text);
            }
            Arg::Long("dry-run") => {
                options.print_mode = PrintMode::DryRun;
            }
            Arg::Long("spool-dir") => {
                let val = parser.value().map_err(|e| e.to_string())?;
                let path = val
                    .into_string()
                    .map_err(|os| format!("invalid UTF-8 argument: {}", os.to_string_lossy()))?;
                options.spool_dir = PathBuf::from(path);
            }
            Arg::Short('p') | Arg::Long("printer") => {
                let val = parser.value().map_err(|e| e.to_string())?;
                let queue = val
                    .into_string()
                    .map_err(|os| format!("invalid UTF-8 argument: {}", os.to_string_lossy()))?;
                options.printer_queue = queue;
            }
            Arg::Short('d') | Arg::Long("device") => {
                let val = parser.value().map_err(|e| e.to_string())?;
                let dev = val
                    .into_string()
                    .map_err(|os| format!("invalid UTF-8 argument: {}", os.to_string_lossy()))?;
                options.usb_device = PathBuf::from(dev);
            }
            Arg::Long("inventory-url") => {
                let val = parser.value().map_err(|e| e.to_string())?;
                let url = val
                    .into_string()
                    .map_err(|os| format!("invalid UTF-8 argument: {}", os.to_string_lossy()))?;
                options.inventory_base_url = url;
            }
            Arg::Long("listen") => {
                let val = parser.value().map_err(|e| e.to_string())?;
                let addr = val
                    .into_string()
                    .map_err(|os| format!("invalid UTF-8 argument: {}", os.to_string_lossy()))?;
                options.listen_addr = addr;
            }
            Arg::Long("daemon") => {
                options.daemon_mode = true;
            }
            Arg::Short('h') | Arg::Long("help") => {
                options.show_help = true;
            }
            other => return Err(other.unexpected().to_string()),
        }
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_args_to_default() {
        let args: Vec<String> = Vec::new();
        let opts = parse_cli_args(&args).expect("empty args should parse");
        assert_eq!(opts.command, None);
        assert_eq!(opts.print_mode, PrintMode::ExecutePrint);
        assert_eq!(opts.spool_dir, PathBuf::from("/tmp/label-agent-spool"));
        assert_eq!(opts.printer_queue, "BT320_Printer");
        assert_eq!(opts.inventory_base_url, DEFAULT_INVENTORY_URL);
        assert!(!opts.daemon_mode);
        assert!(!opts.show_help);
    }

    #[test]
    fn parses_command_and_dry_run() {
        let args = vec![
            "-c".to_owned(),
            "print box label for BOX-042".to_owned(),
            "--dry-run".to_owned(),
        ];
        let opts = parse_cli_args(&args).expect("should parse command and dry-run");
        assert_eq!(opts.command.as_deref(), Some("print box label for BOX-042"));
        assert_eq!(opts.print_mode, PrintMode::DryRun);
    }

    #[test]
    fn parses_flags_and_long_options() {
        let args = vec![
            "--command".to_owned(),
            "what is inside box BOX-042".to_owned(),
            "--spool-dir".to_owned(),
            "/var/spool/labels".to_owned(),
            "--printer".to_owned(),
            "Brother_QL_800".to_owned(),
            "--device".to_owned(),
            "/dev/usb/lp0".to_owned(),
            "--inventory-url".to_owned(),
            "https://inventory.molinadeveloping.com".to_owned(),
            "--listen".to_owned(),
            "127.0.0.1:9090".to_owned(),
            "--daemon".to_owned(),
        ];
        let opts = parse_cli_args(&args).expect("should parse all long flags");
        assert_eq!(opts.command.as_deref(), Some("what is inside box BOX-042"));
        assert_eq!(opts.spool_dir, PathBuf::from("/var/spool/labels"));
        assert_eq!(opts.printer_queue, "Brother_QL_800");
        assert_eq!(opts.usb_device, PathBuf::from("/dev/usb/lp0"));
        assert_eq!(
            opts.inventory_base_url,
            "https://inventory.molinadeveloping.com"
        );
        assert_eq!(opts.listen_addr, "127.0.0.1:9090");
        assert!(opts.daemon_mode);
    }

    #[test]
    fn parses_help_flag() {
        let args = vec!["-h".to_owned()];
        let opts = parse_cli_args(&args).expect("should parse help flag");
        assert!(opts.show_help);
        assert!(help_banner().contains("Usage: label-agent"));
    }

    #[test]
    fn rejects_unknown_flag() {
        let args = vec!["--unknown-flag".to_owned()];
        let result = parse_cli_args(&args);
        assert!(result.is_err());
    }
}
