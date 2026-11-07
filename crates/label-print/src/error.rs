//! Hardware print error definitions and taxonomy for Label-Agent.

use std::io;
use thiserror::Error;

/// Error enumeration for print job creation, validation, and hardware
/// transport execution.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PrintError {
    /// The specified print queue name was empty or whitespace only.
    #[error("print queue name cannot be empty")]
    EmptyQueueName,

    /// The number of copies requested was zero.
    #[error("copy count must be greater than zero")]
    ZeroCopies,

    /// The spool file to print was not found on disk.
    #[error("missing spool file at: {path}")]
    MissingSpoolFile {
        /// File path to the missing spool artifact.
        path: String,
    },

    /// The spooler binary (such as `/usr/bin/lpr`) was not found or is
    /// not executable.
    #[error("spooler binary not found at: {path}")]
    SpoolerBinaryNotFound {
        /// File path where the spooler binary was expected.
        path: String,
    },

    /// The print command failed with a non-zero exit status or error output.
    #[error("print command failed with status {status}: {stderr}")]
    CommandFailed {
        /// Process exit status code.
        status: i32,
        /// Standard error output captured from the failed command.
        stderr: String,
    },

    /// An I/O error occurred while preparing or dispatching the print job.
    #[error("I/O error: {message}")]
    Io {
        /// Descriptive error message detailing the I/O failure.
        message: String,
    },
}

impl From<io::Error> for PrintError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_queue_name_display() {
        let err = PrintError::EmptyQueueName;
        assert_eq!(err.to_string(), "print queue name cannot be empty");
    }

    #[test]
    fn test_zero_copies_display() {
        let err = PrintError::ZeroCopies;
        assert_eq!(err.to_string(), "copy count must be greater than zero");
    }

    #[test]
    fn test_missing_spool_file_display() {
        let err = PrintError::MissingSpoolFile {
            path: "/tmp/labels/box1.pdf".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "missing spool file at: /tmp/labels/box1.pdf"
        );
    }

    #[test]
    fn test_spooler_binary_not_found_display() {
        let err = PrintError::SpoolerBinaryNotFound {
            path: "/usr/bin/lpr".to_owned(),
        };
        assert_eq!(err.to_string(), "spooler binary not found at: /usr/bin/lpr");
    }

    #[test]
    fn test_command_failed_display() {
        let err = PrintError::CommandFailed {
            status: 1,
            stderr: "queue does not exist".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "print command failed with status 1: queue does not exist"
        );
    }

    #[test]
    fn test_io_display() {
        let err = PrintError::Io {
            message: "permission denied".to_owned(),
        };
        assert_eq!(err.to_string(), "I/O error: permission denied");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: PrintError = io_err.into();
        assert_eq!(
            err,
            PrintError::Io {
                message: "file not found".to_owned()
            }
        );
    }
}
