//! CUPS (`lpr`) command-line print transport for Label-Agent.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use label_core::PrintMode;

use crate::error::PrintError;
use crate::transport::PrintJob;
use crate::transport::PrintReceipt;
use crate::transport::PrintTransport;

/// Hardware print transport invoking CUPS `lpr` CLI binary.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CupsTransport {
    lpr_binary: PathBuf,
    default_queue: String,
}

impl CupsTransport {
    /// Creates a new `CupsTransport` with an explicit `lpr` binary path and queue.
    #[must_use]
    pub fn new(lpr_binary: PathBuf, default_queue: String) -> Self {
        Self {
            lpr_binary,
            default_queue,
        }
    }

    /// Returns the configured path to the `lpr` binary.
    #[must_use]
    pub fn lpr_binary(&self) -> &Path {
        &self.lpr_binary
    }

    /// Returns the default queue name.
    #[must_use]
    pub fn default_queue(&self) -> &str {
        &self.default_queue
    }

    /// Builds command-line arguments for invoking `lpr` with the given `PrintJob`.
    #[must_use]
    pub fn build_lpr_args(&self, job: &PrintJob) -> Vec<String> {
        vec![
            "-P".to_owned(),
            job.queue_name().to_owned(),
            "-#".to_owned(),
            job.copies().to_string(),
            job.artifact_path().display().to_string(),
        ]
    }
}

impl Default for CupsTransport {
    fn default() -> Self {
        Self::new(
            PathBuf::from("/usr/bin/lpr"),
            "Dymo_LabelWriter_450".to_owned(),
        )
    }
}

impl PrintTransport for CupsTransport {
    fn dispatch(&self, job: &PrintJob) -> Result<PrintReceipt, PrintError> {
        let artifact = job.artifact_path();
        if !artifact.exists() {
            return Err(PrintError::MissingSpoolFile {
                path: artifact.display().to_string(),
            });
        }

        if !self.lpr_binary.exists() {
            return Err(PrintError::SpoolerBinaryNotFound {
                path: self.lpr_binary.display().to_string(),
            });
        }

        let args = self.build_lpr_args(job);
        let output = Command::new(&self.lpr_binary)
            .args(&args)
            .output()
            .map_err(|err| PrintError::Io {
                message: err.to_string(),
            })?;

        if !output.status.success() {
            let status = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(PrintError::CommandFailed { status, stderr });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let job_id = if stdout.is_empty() {
            format!("cups-{}", job.queue_name())
        } else {
            stdout
        };

        Ok(PrintReceipt::new(
            job_id,
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
    fn test_default_cups_transport() {
        let transport = CupsTransport::default();
        assert_eq!(transport.lpr_binary(), Path::new("/usr/bin/lpr"));
        assert_eq!(transport.default_queue(), "Dymo_LabelWriter_450");
    }

    #[test]
    fn test_build_lpr_args() {
        let transport = CupsTransport::default();
        let job = PrintJob::new(
            "Brother_QL_800",
            PathBuf::from("/tmp/label-agent/BOX-042.lbx"),
            LabelFormat::BrotherPtouchLbx,
            3,
        )
        .expect("valid job");

        let args = transport.build_lpr_args(&job);
        assert_eq!(
            args,
            vec![
                "-P".to_owned(),
                "Brother_QL_800".to_owned(),
                "-#".to_owned(),
                "3".to_owned(),
                "/tmp/label-agent/BOX-042.lbx".to_owned(),
            ]
        );
    }

    #[test]
    fn test_missing_spool_file_error() {
        let transport = CupsTransport::default();
        let job = PrintJob::new(
            "Dymo_LabelWriter_450",
            PathBuf::from("/tmp/nonexistent_file_path_12345.pdf"),
            LabelFormat::Dymo30256Pdf,
            1,
        )
        .expect("valid job");

        let err = transport.dispatch(&job);
        match err {
            Err(PrintError::MissingSpoolFile { path }) => {
                assert!(path.contains("nonexistent_file_path_12345.pdf"));
            }
            other => panic!("expected MissingSpoolFile error, got {other:?}"),
        }
    }

    #[test]
    fn test_missing_lpr_binary_error() {
        let transport = CupsTransport::new(
            PathBuf::from("/nonexistent/bin/lpr"),
            "Dymo_LabelWriter_450".to_owned(),
        );
        let existing_path = PathBuf::from("/etc/hosts");
        let job = PrintJob::new(
            "Dymo_LabelWriter_450",
            existing_path,
            LabelFormat::Dymo30256Pdf,
            1,
        )
        .expect("valid job");

        let err = transport.dispatch(&job);
        match err {
            Err(PrintError::SpoolerBinaryNotFound { path }) => {
                assert_eq!(path, "/nonexistent/bin/lpr");
            }
            other => panic!("expected SpoolerBinaryNotFound error, got {other:?}"),
        }
    }
}
