//! Monochrome 1-bit Windows BMP encoder for Brother LBX and thermal print
//! spools.

use crate::error::RenderError;
use crate::qr::QrMatrix;

const BMP_INFO_HEADER_SIZE: u32 = 40;
const BMP_PIXEL_OFFSET: u32 = 62;
const BMP_DIMENSION: u32 = 192;
const BMP_BYTES_PER_ROW: usize = 24; // 192 bits / 8 bits = 24 bytes
const BMP_IMAGE_SIZE: u32 = 4608; // 192 rows * 24 bytes = 4608
const BMP_TOTAL_FILE_SIZE: u32 = 4670; // 62 + 4608 = 4670

/// Encodes a QR payload into a 192x192 1-bit uncompressed Windows BMP image.
///
/// The BMP structure consists of:
/// - 14-byte `BITMAPFILEHEADER` (`BM`, size 4670, offset 62)
/// - 40-byte `BITMAPINFOHEADER` (size 40, width 192, height 192, 1-bit,
///   uncompressed)
/// - 8-byte color table (index 0 = black, index 1 = white)
/// - 4608 bytes of pixel data (192 rows * 24 bytes/row, bottom-up)
///
/// # Errors
///
/// Returns [`RenderError`] if QR encoding fails or the payload is empty.
pub fn encode_monochrome_bmp_192(qr_payload: &str) -> Result<Vec<u8>, RenderError> {
    let qr = QrMatrix::encode(qr_payload)?;
    let module_count = qr.width();

    let mut bytes = Vec::with_capacity(BMP_TOTAL_FILE_SIZE as usize);

    // 1. BITMAPFILEHEADER (14 bytes)
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&BMP_TOTAL_FILE_SIZE.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes()); // Reserved (4 bytes)
    bytes.extend_from_slice(&BMP_PIXEL_OFFSET.to_le_bytes());

    // 2. BITMAPINFOHEADER (40 bytes)
    bytes.extend_from_slice(&BMP_INFO_HEADER_SIZE.to_le_bytes());
    bytes.extend_from_slice(&(BMP_DIMENSION as i32).to_le_bytes());
    bytes.extend_from_slice(&(BMP_DIMENSION as i32).to_le_bytes()); // bottom-up
    bytes.extend_from_slice(&1u16.to_le_bytes()); // planes = 1
    bytes.extend_from_slice(&1u16.to_le_bytes()); // bit count = 1
    bytes.extend_from_slice(&0u32.to_le_bytes()); // compression = BI_RGB (0)
    bytes.extend_from_slice(&BMP_IMAGE_SIZE.to_le_bytes());
    bytes.extend_from_slice(&2835i32.to_le_bytes()); // X pixels per meter (~72 DPI)
    bytes.extend_from_slice(&2835i32.to_le_bytes()); // Y pixels per meter (~72 DPI)
    bytes.extend_from_slice(&2u32.to_le_bytes()); // colors used = 2
    bytes.extend_from_slice(&2u32.to_le_bytes()); // important colors = 2

    // 3. Color Table (8 bytes): index 0 = black, index 1 = white
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // index 0: black (B, G, R, 0)
    bytes.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0x00]); // index 1: white (B, G, R, 0)

    // 4. Pixel data (4608 bytes): 192 rows * 24 bytes/row, bottom-up
    // Default all pixels to 1 (white)
    let mut pixel_data = vec![0xFFu8; BMP_IMAGE_SIZE as usize];

    let mut scale = (192 / (module_count + 8)).max(1);
    if module_count * scale > 192 && module_count > 0 {
        scale = (192 / module_count).max(1);
    }
    let qr_size = module_count * scale;
    let offset_x = (192_usize.saturating_sub(qr_size)) / 2;
    let offset_y = (192_usize.saturating_sub(qr_size)) / 2;

    for my in 0..module_count {
        for mx in 0..module_count {
            if qr.is_dark(mx, my) {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = offset_x + mx * scale + dx;
                        let py = offset_y + my * scale + dy;
                        if px < 192 && py < 192 {
                            let row = 191 - py;
                            let byte_idx = row * BMP_BYTES_PER_ROW + px / 8;
                            let bit_pos = 7 - (px % 8);
                            // Clear bit to 0 (index 0 = black)
                            pixel_data[byte_idx] &= !(1 << bit_pos);
                        }
                    }
                }
            }
        }
    }

    bytes.extend_from_slice(&pixel_data);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmp_192_structure() {
        let bytes = encode_monochrome_bmp_192("BOX-042").expect("monochrome BMP encode succeeds");

        assert_eq!(bytes.len(), 4670);
        assert_eq!(&bytes[0..2], b"BM");

        let file_size = u32::from_le_bytes(bytes[2..6].try_into().expect("slice 2..6 has 4 bytes"));
        assert_eq!(file_size, 4670);

        let pixel_offset =
            u32::from_le_bytes(bytes[10..14].try_into().expect("slice 10..14 has 4 bytes"));
        assert_eq!(pixel_offset, 62);

        let info_header_size =
            u32::from_le_bytes(bytes[14..18].try_into().expect("slice 14..18 has 4 bytes"));
        assert_eq!(info_header_size, 40);

        let width = i32::from_le_bytes(bytes[18..22].try_into().expect("slice 18..22 has 4 bytes"));
        assert_eq!(width, 192);

        let height =
            i32::from_le_bytes(bytes[22..26].try_into().expect("slice 22..26 has 4 bytes"));
        assert_eq!(height, 192);

        let planes =
            u16::from_le_bytes(bytes[26..28].try_into().expect("slice 26..28 has 2 bytes"));
        assert_eq!(planes, 1);

        let bit_count =
            u16::from_le_bytes(bytes[28..30].try_into().expect("slice 28..30 has 2 bytes"));
        assert_eq!(bit_count, 1);
    }

    #[test]
    fn test_bmp_empty_payload_fails() {
        let result = encode_monochrome_bmp_192("   ");
        assert_eq!(result, Err(RenderError::EmptyPayload));
    }
}
