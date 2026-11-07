//! Spool directory manager and path sanitization for rendered label
//! artifacts.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::error::RenderError;

/// Spool directory manager that safely writes artifacts to a designated spool
/// location.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoolManager {
    spool_dir: PathBuf,
}

impl Default for SpoolManager {
    fn default() -> Self {
        Self {
            spool_dir: PathBuf::from("/tmp/label-agent"),
        }
    }
}

impl SpoolManager {
    /// Creates a new spool manager bound to the given directory path.
    #[must_use]
    pub fn new(spool_dir: PathBuf) -> Self {
        Self { spool_dir }
    }

    /// Returns a reference to the underlying spool directory path.
    #[must_use]
    pub fn spool_dir(&self) -> &Path {
        &self.spool_dir
    }

    /// Sanitizes an input identifier into a safe filename stem.
    ///
    /// Replaces characters other than ASCII alphanumeric or `-` with `_`,
    /// collapses consecutive `_` into a single `_`, and trims leading/trailing `_`.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::InvalidSpoolIdentifier`] if the resulting stem is
    /// empty or contains only `-` and `_`.
    pub fn sanitize_stem(label_id: &str) -> Result<String, RenderError> {
        let mut replaced = String::with_capacity(label_id.len());
        for c in label_id.chars() {
            if c.is_ascii_alphanumeric() || c == '-' {
                replaced.push(c);
            } else {
                replaced.push('_');
            }
        }

        // Collapse consecutive underscores
        let mut collapsed = String::with_capacity(replaced.len());
        let mut last_was_underscore = false;
        for c in replaced.chars() {
            if c == '_' {
                if !last_was_underscore {
                    collapsed.push('_');
                    last_was_underscore = true;
                }
            } else {
                collapsed.push(c);
                last_was_underscore = false;
            }
        }

        // Trim leading and trailing underscores
        let trimmed = collapsed.trim_matches('_');

        // Check if empty or contains only '-' and '_'
        let has_alnum = trimmed.chars().any(|c| c.is_ascii_alphanumeric());
        if !has_alnum {
            return Err(RenderError::InvalidSpoolIdentifier);
        }

        Ok(trimmed.to_owned())
    }

    /// Writes raw artifact bytes to `<spool_dir>/<sanitized_stem>.<sanitized_ext>`.
    ///
    /// Creates the spool directory and any missing parent directories if absent.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::InvalidSpoolIdentifier`] if the label ID or extension
    /// cannot be sanitized, or [`RenderError::Io`] if directory creation or file
    /// writing fails.
    pub fn write_artifact(
        &self,
        label_id: &str,
        extension: &str,
        bytes: &[u8],
    ) -> Result<PathBuf, RenderError> {
        let stem = Self::sanitize_stem(label_id)?;
        let clean_ext = extension.trim_start_matches('.');
        let ext = Self::sanitize_stem(clean_ext)?;

        if !self.spool_dir.exists() {
            fs::create_dir_all(&self.spool_dir).map_err(|err| RenderError::Io {
                message: format!(
                    "failed to create spool directory {}: {err}",
                    self.spool_dir.display()
                ),
            })?;
        }

        let file_name = format!("{stem}.{ext}");
        let dest_path = self.spool_dir.join(file_name);

        fs::write(&dest_path, bytes).map_err(|err| RenderError::Io {
            message: format!("failed to write artifact {}: {err}", dest_path.display()),
        })?;

        Ok(dest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_sanitize_stem_valid() {
        let stem = SpoolManager::sanitize_stem("BOX-001/../../etc").expect("valid sanitized stem");
        assert_eq!(stem, "BOX-001_etc");

        let stem2 =
            SpoolManager::sanitize_stem("BOX 042 (Hardware)").expect("valid sanitized stem");
        assert_eq!(stem2, "BOX_042_Hardware");
    }

    #[test]
    fn test_sanitize_stem_invalid() {
        assert_eq!(
            SpoolManager::sanitize_stem(""),
            Err(RenderError::InvalidSpoolIdentifier)
        );
        assert_eq!(
            SpoolManager::sanitize_stem("   "),
            Err(RenderError::InvalidSpoolIdentifier)
        );
        assert_eq!(
            SpoolManager::sanitize_stem("---"),
            Err(RenderError::InvalidSpoolIdentifier)
        );
        assert_eq!(
            SpoolManager::sanitize_stem("../../"),
            Err(RenderError::InvalidSpoolIdentifier)
        );
        assert_eq!(
            SpoolManager::sanitize_stem("___---___"),
            Err(RenderError::InvalidSpoolIdentifier)
        );
    }

    #[test]
    fn test_write_artifact_in_temp_dir() {
        let temp_dir = env::temp_dir().join("label_agent_test_spool");
        let manager = SpoolManager::new(temp_dir.clone());
        assert_eq!(manager.spool_dir(), temp_dir.as_path());

        let payload = b"fake artifact content";
        let path = manager
            .write_artifact("BOX-001/../../etc", ".bmp", payload)
            .expect("artifact write succeeds");

        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some("BOX-001_etc.bmp")
        );
        assert!(path.exists());

        let content = fs::read(&path).expect("read written artifact");
        assert_eq!(content, payload);

        // Cleanup
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&temp_dir);
    }
}
