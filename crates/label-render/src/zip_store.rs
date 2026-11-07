//! Pure-Rust uncompressed (STORE, compression method 0) PKZIP archive builder.
//!
//! Provides [`ZipStoreBuilder`] for assembling valid `.zip`, `.odt`, and
//! `.lbx` packages in memory without external compression libraries.

/// Computes the standard IEEE 802.3 32-bit Cyclic Redundancy Check (CRC-32).
///
/// Uses the canonical reflected polynomial `0xEDB88320` with standard initial
/// remainder `0xFFFFFFFF` and inverted final remainder.
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[derive(Debug, Clone)]
struct ZipFileEntry {
    name: String,
    data: Vec<u8>,
    crc: u32,
    offset: u32,
}

/// In-memory uncompressed (`STORE`, method `0`) PKZIP archive builder.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ZipStoreBuilder {
    entries: Vec<ZipFileEntry>,
}

impl ZipStoreBuilder {
    /// Creates a new, empty [`ZipStoreBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an uncompressed file entry with the specified name and payload.
    pub fn add_file(&mut self, name: &str, data: &[u8]) {
        self.entries.push(ZipFileEntry {
            name: name.to_string(),
            data: data.to_vec(),
            crc: crc32(data),
            offset: 0,
        });
    }

    /// Finalizes the archive, writing Local File Headers, raw file payloads,
    /// Central Directory Headers, and the End of Central Directory record.
    #[must_use]
    pub fn finish(mut self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // 1. Write Local File Headers and file payloads
        for entry in &mut self.entries {
            entry.offset = buffer.len() as u32;

            // Local file header signature: 0x04034b50 ("PK\x03\x04")
            buffer.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
            // Version needed to extract: 20 (PKZIP 2.0)
            buffer.extend_from_slice(&20u16.to_le_bytes());
            // General purpose bit flag: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Compression method: 0 (STORE)
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Last mod file time: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Last mod file date: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // CRC-32 checksum
            buffer.extend_from_slice(&entry.crc.to_le_bytes());
            // Compressed size
            buffer.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            // Uncompressed size
            buffer.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            // File name length
            buffer.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
            // Extra field length
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // File name bytes
            buffer.extend_from_slice(entry.name.as_bytes());
            // Raw file data
            buffer.extend_from_slice(&entry.data);
        }

        let cd_start = buffer.len() as u32;

        // 2. Write Central Directory Headers
        for entry in &self.entries {
            // Central directory header signature: 0x02014b50 ("PK\x01\x02")
            buffer.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
            // Version made by: 20
            buffer.extend_from_slice(&20u16.to_le_bytes());
            // Version needed to extract: 20
            buffer.extend_from_slice(&20u16.to_le_bytes());
            // General purpose bit flag: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Compression method: 0 (STORE)
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Last mod file time: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Last mod file date: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // CRC-32 checksum
            buffer.extend_from_slice(&entry.crc.to_le_bytes());
            // Compressed size
            buffer.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            // Uncompressed size
            buffer.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            // File name length
            buffer.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
            // Extra field length: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // File comment length: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Disk number start: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // Internal file attributes: 0
            buffer.extend_from_slice(&0u16.to_le_bytes());
            // External file attributes: 0
            buffer.extend_from_slice(&0u32.to_le_bytes());
            // Relative offset of local header
            buffer.extend_from_slice(&entry.offset.to_le_bytes());
            // File name bytes
            buffer.extend_from_slice(entry.name.as_bytes());
        }

        let cd_size = (buffer.len() as u32) - cd_start;
        let total_entries = self.entries.len() as u16;

        // 3. Write End of Central Directory Record (EOCD)
        // EOCD signature: 0x06054b50 ("PK\x05\x06")
        buffer.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        // Number of this disk: 0
        buffer.extend_from_slice(&0u16.to_le_bytes());
        // Disk where central directory starts: 0
        buffer.extend_from_slice(&0u16.to_le_bytes());
        // Number of central directory records on this disk
        buffer.extend_from_slice(&total_entries.to_le_bytes());
        // Total number of central directory records
        buffer.extend_from_slice(&total_entries.to_le_bytes());
        // Size of central directory in bytes
        buffer.extend_from_slice(&cd_size.to_le_bytes());
        // Offset of start of central directory
        buffer.extend_from_slice(&cd_start.to_le_bytes());
        // Comment length: 0
        buffer.extend_from_slice(&0u16.to_le_bytes());

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_standard_vectors() {
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn test_zip_store_empty() {
        let builder = ZipStoreBuilder::new();
        let bytes = builder.finish();
        // EOCD record is 22 bytes
        assert_eq!(bytes.len(), 22);
        assert_eq!(&bytes[0..4], &0x0605_4b50u32.to_le_bytes());
    }

    #[test]
    fn test_zip_store_single_file() {
        let mut builder = ZipStoreBuilder::new();
        builder.add_file("test.txt", b"Hello, World!");
        let bytes = builder.finish();

        // Local header magic
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        // Filename in local header (offset 30)
        assert_eq!(&bytes[30..38], b"test.txt");
        // Data in local header (offset 38)
        assert_eq!(&bytes[38..51], b"Hello, World!");
    }

    #[test]
    fn test_zip_store_multiple_files_ordering() {
        let mut builder = ZipStoreBuilder::new();
        builder.add_file("first.bin", b"111");
        builder.add_file("second.bin", b"2222");
        let bytes = builder.finish();

        // First local header magic and name
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        assert_eq!(&bytes[30..39], b"first.bin");
        assert_eq!(&bytes[39..42], b"111");

        // Second local header offset: 30 + 9 + 3 = 42
        assert_eq!(&bytes[42..46], b"PK\x03\x04");
        assert_eq!(&bytes[72..82], b"second.bin");
        assert_eq!(&bytes[82..86], b"2222");
    }
}
