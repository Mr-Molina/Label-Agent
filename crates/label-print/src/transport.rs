//! Hardware print transport abstractions, print jobs, receipts, and dry-run
//! transport.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use label_core::LabelFormat;
use label_core::PrintMode;

use crate::error::PrintError;

/// A submitted print job targeting a specific hardware or CUPS printer queue.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintJob {
    queue_name: String,
    artifact_path: PathBuf,
    format: LabelFormat,
    copies: u8,
}

impl PrintJob {
    /// Creates a new `PrintJob`.
    ///
    /// # Errors
    /// Returns `PrintError::EmptyQueueName` if `queue_name` is empty or only
    /// whitespace.
    /// Returns `PrintError::ZeroCopies` if `copies == 0`.
    pub fn new(
        queue_name: &str,
        artifact_path: PathBuf,
        format: LabelFormat,
        copies: u8,
    ) -> Result<Self, PrintError> {
        let trimmed_queue = queue_name.trim();
        if trimmed_queue.is_empty() {
            return Err(PrintError::EmptyQueueName);
        }
        if copies == 0 {
            return Err(PrintError::ZeroCopies);
        }
        Ok(Self {
            queue_name: trimmed_queue.to_owned(),
            artifact_path,
            format,
            copies,
        })
    }

    /// Returns the target printer queue name.
    #[must_use]
    pub fn queue_name(&self) -> &str {
        &self.queue_name
    }

    /// Returns the path to the spooled label artifact.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns the label format of the spooled artifact.
    #[must_use]
    pub const fn format(&self) -> LabelFormat {
        self.format
    }

    /// Returns the requested copy count.
    #[must_use]
    pub const fn copies(&self) -> u8 {
        self.copies
    }
}

/// A receipt returned after dispatching a print job to a transport.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintReceipt {
    job_id: String,
    queue_name: String,
    mode: PrintMode,
}

impl PrintReceipt {
    /// Creates a new `PrintReceipt`.
    #[must_use]
    pub fn new(job_id: String, queue_name: String, mode: PrintMode) -> Self {
        Self {
            job_id,
            queue_name,
            mode,
        }
    }

    /// Returns the identifier assigned by the spooler or mock transport.
    #[must_use]
    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    /// Returns the printer queue to which the job was dispatched.
    #[must_use]
    pub fn queue_name(&self) -> &str {
        &self.queue_name
    }

    /// Returns whether the job was dispatched in live or dry-run mode.
    #[must_use]
    pub const fn mode(&self) -> PrintMode {
        self.mode
    }
}

/// Trait for dispatching label print jobs to hardware printers or simulations.
pub trait PrintTransport {
    /// Dispatches a print job to the underlying printer or simulation buffer.
    ///
    /// # Errors
    /// Returns `PrintError` if dispatch fails, the file is missing, or the
    /// command errors.
    fn dispatch(&self, job: &PrintJob) -> Result<PrintReceipt, PrintError>;
}

/// In-memory dry-run print transport that records dispatched jobs without
/// physical output.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct DryRunTransport {
    jobs: Mutex<Vec<PrintJob>>,
}

impl DryRunTransport {
    /// Creates a new in-memory `DryRunTransport`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(Vec::new()),
        }
    }

    /// Executes a closure with a read-only slice of recorded print jobs.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned.
    pub fn with_jobs<R>(&self, f: impl FnOnce(&[PrintJob]) -> R) -> R {
        let guard = self
            .jobs
            .lock()
            .expect("dry-run jobs mutex is never poisoned");
        f(&guard)
    }
}

impl PrintTransport for DryRunTransport {
    fn dispatch(&self, job: &PrintJob) -> Result<PrintReceipt, PrintError> {
        let mut guard = self
            .jobs
            .lock()
            .expect("dry-run jobs mutex is never poisoned");
        let job_number = guard.len() + 1;
        guard.push(job.clone());
        Ok(PrintReceipt::new(
            format!("dryrun-{job_number}"),
            job.queue_name().to_owned(),
            PrintMode::DryRun,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_job_validation() {
        let valid_path = PathBuf::from("/tmp/test.pdf");
        let job = PrintJob::new("Dymo", valid_path.clone(), LabelFormat::Dymo30256Pdf, 1)
            .expect("valid job creation");
        assert_eq!(job.queue_name(), "Dymo");
        assert_eq!(job.artifact_path(), valid_path.as_path());
        assert_eq!(job.format(), LabelFormat::Dymo30256Pdf);
        assert_eq!(job.copies(), 1);

        assert_eq!(
            PrintJob::new("", valid_path.clone(), LabelFormat::Dymo30256Pdf, 1),
            Err(PrintError::EmptyQueueName)
        );
        assert_eq!(
            PrintJob::new("   ", valid_path.clone(), LabelFormat::Dymo30256Pdf, 1),
            Err(PrintError::EmptyQueueName)
        );
        assert_eq!(
            PrintJob::new("Dymo", valid_path, LabelFormat::Dymo30256Pdf, 0),
            Err(PrintError::ZeroCopies)
        );
    }

    #[test]
    fn test_print_receipt_getters() {
        let receipt = PrintReceipt::new(
            "job-42".to_owned(),
            "Brother_QL_800".to_owned(),
            PrintMode::ExecutePrint,
        );
        assert_eq!(receipt.job_id(), "job-42");
        assert_eq!(receipt.queue_name(), "Brother_QL_800");
        assert_eq!(receipt.mode(), PrintMode::ExecutePrint);
    }

    #[test]
    fn test_dry_run_transport_dispatch() {
        let transport = DryRunTransport::new();
        let job = PrintJob::new(
            "TestQueue",
            PathBuf::from("/tmp/test.odt"),
            LabelFormat::BoxLabelsOdt,
            2,
        )
        .expect("valid job");

        let receipt = transport.dispatch(&job).expect("dispatch succeeds");
        assert_eq!(receipt.job_id(), "dryrun-1");
        assert_eq!(receipt.queue_name(), "TestQueue");
        assert_eq!(receipt.mode(), PrintMode::DryRun);

        let count = transport.with_jobs(|jobs| {
            assert_eq!(jobs.len(), 1);
            assert_eq!(jobs[0].copies(), 2);
            jobs.len()
        });
        assert_eq!(count, 1);
    }
}
