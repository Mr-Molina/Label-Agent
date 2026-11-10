//! Direct USB character device (`usblp`) print transport for thermal label
//! printers such as the Jadens / Phomemo BT320.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use label_core::PrintMode;

use crate::error::PrintError;
use crate::transport::PrintJob;
use crate::transport::PrintReceipt;
use crate::transport::PrintTransport;

/// Default persistent udev symlink for the Jadens BT320 thermal printer.
pub const DEFAULT_BT320_DEVICE: &str = "/dev/bt320-printer";

/// Fallback kernel USB printer character device node.
pub const FALLBACK_USB_LP_DEVICE: &str = "/dev/usb/lp0";

/// Hardware print transport writing raw spool bytes directly to a USB printer
/// character device (`/dev/usb/lp*` or persistent udev symlink).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbLpTransport {
    device_path: PathBuf,
}

impl UsbLpTransport {
    /// Creates a new `UsbLpTransport` targeting the specified device path.
    #[must_use]
    pub fn new(device_path: PathBuf) -> Self {
        Self { device_path }
    }

    /// Returns the target device path.
    #[must_use]
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }
}

impl Default for UsbLpTransport {
    fn default() -> Self {
        Self::new(PathBuf::from(DEFAULT_BT320_DEVICE))
    }
}

impl PrintTransport for UsbLpTransport {
    fn dispatch(&self, job: &PrintJob) -> Result<PrintReceipt, PrintError> {
        if job.mode() == PrintMode::DryRun {
            return Ok(PrintReceipt::new(
                format!("dryrun-{}", job.queue_name()),
                job.queue_name().to_owned(),
                PrintMode::DryRun,
            ));
        }

        let artifact = job.artifact_path();
        if !artifact.exists() {
            return Err(PrintError::MissingSpoolFile {
                path: artifact.display().to_string(),
            });
        }

        let bytes = std::fs::read(artifact).map_err(|err| PrintError::Io {
            message: format!(
                "failed to read spool artifact {}: {err}",
                artifact.display()
            ),
        })?;

        let mut device_file = OpenOptions::new()
            .write(true)
            .open(&self.device_path)
            .map_err(|err| PrintError::Io {
                message: format!(
                    "failed to open printer device {}: {err}",
                    self.device_path.display()
                ),
            })?;

        for _ in 0..job.copies() {
            device_file
                .write_all(&bytes)
                .map_err(|err| PrintError::Io {
                    message: format!(
                        "failed to write spool bytes to device {}: {err}",
                        self.device_path.display()
                    ),
                })?;
        }

        device_file.sync_all().map_err(|err| PrintError::Io {
            message: format!(
                "failed to sync printer device {}: {err}",
                self.device_path.display()
            ),
        })?;

        // Explicitly close the file handle before unlinking the spool file.
        drop(device_file);

        std::fs::remove_file(artifact).map_err(|err| PrintError::Io {
            message: format!("failed to remove spool file {}: {err}", artifact.display()),
        })?;

        Ok(PrintReceipt::new(
            format!("usblp-{}", job.queue_name()),
            job.queue_name().to_owned(),
            PrintMode::ExecutePrint,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use label_core::LabelFormat;

    #[test]
    fn test_constants_and_default() {
        assert_eq!(DEFAULT_BT320_DEVICE, "/dev/bt320-printer");
        assert_eq!(FALLBACK_USB_LP_DEVICE, "/dev/usb/lp0");

        let default_transport = UsbLpTransport::default();
        assert_eq!(
            default_transport.device_path(),
            Path::new(DEFAULT_BT320_DEVICE)
        );
    }

    #[test]
    fn test_dry_run_dispatch() {
        let temp_dir = std::env::temp_dir();
        let placeholder_path = temp_dir.join("nonexistent_spool_dryrun.bin");
        let transport = UsbLpTransport::new(PathBuf::from("/dev/null"));

        let job = PrintJob::new_with_mode(
            "BT320-Dry",
            placeholder_path,
            LabelFormat::Dymo30256Pdf,
            1,
            PrintMode::DryRun,
        )
        .expect("valid dry-run job");

        let receipt = transport.dispatch(&job).expect("dry run dispatch");
        assert_eq!(receipt.job_id(), "dryrun-BT320-Dry");
        assert_eq!(receipt.queue_name(), "BT320-Dry");
        assert_eq!(receipt.mode(), PrintMode::DryRun);
    }

    #[test]
    fn test_live_dispatch_writes_and_removes_spool() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(12345);

        let spool_path = temp_dir.join(format!("spool_{unique_id}.bin"));
        let dev_path = temp_dir.join(format!("dev_{unique_id}.bin"));

        std::fs::write(&spool_path, b"TEST-LABEL-DATA").expect("write spool");
        std::fs::write(&dev_path, b"").expect("create dev placeholder");

        let transport = UsbLpTransport::new(dev_path.clone());
        let job = PrintJob::new(
            "BT320",
            spool_path.clone(),
            LabelFormat::BrotherPtouchLbx,
            2,
        )
        .expect("valid job");

        let receipt = transport.dispatch(&job).expect("dispatch succeeds");
        assert_eq!(receipt.queue_name(), "BT320");
        assert_eq!(receipt.mode(), PrintMode::ExecutePrint);

        // Spool file must be unlinked.
        assert!(!spool_path.exists());

        // Device file must contain 2 copies of payload.
        let written = std::fs::read(&dev_path).expect("read device");
        assert_eq!(written, b"TEST-LABEL-DATATEST-LABEL-DATA");

        // Clean up mock device.
        let _ = std::fs::remove_file(dev_path);
    }

    #[test]
    fn test_missing_spool_file_error() {
        let temp_dir = std::env::temp_dir();
        let missing_path = temp_dir.join("nonexistent_spool_12345.bin");
        let transport = UsbLpTransport::default();

        let job =
            PrintJob::new("BT320", missing_path, LabelFormat::Dymo30256Pdf, 1).expect("valid job");

        let result = transport.dispatch(&job);
        match result {
            Err(PrintError::MissingSpoolFile { path }) => {
                assert!(path.contains("nonexistent_spool_12345.bin"));
            }
            other => panic!("expected MissingSpoolFile error, got {other:?}"),
        }
    }

    #[test]
    fn test_missing_device_error() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(67890);

        let spool_path = temp_dir.join(format!("spool_nodev_{unique_id}.bin"));
        let missing_dev = temp_dir.join(format!("missing_dev_{unique_id}.bin"));

        std::fs::write(&spool_path, b"TEST-PAYLOAD").expect("write spool");

        let transport = UsbLpTransport::new(missing_dev);
        let job = PrintJob::new("BT320", spool_path.clone(), LabelFormat::Dymo30256Pdf, 1)
            .expect("valid job");

        let result = transport.dispatch(&job);
        match result {
            Err(PrintError::Io { message }) => {
                assert!(message.contains("failed to open printer device"));
            }
            other => panic!("expected Io error, got {other:?}"),
        }

        // Spool file should NOT be removed when dispatch fails.
        assert!(spool_path.exists());
        let _ = std::fs::remove_file(spool_path);
    }
}
