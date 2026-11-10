//! Pure-Rust QR code, 1-bit BMP, ODT, Brother LBX, and Dymo 30256 PDF
//! label compiler and RAM (`tmpfs`) spooler for Label-Agent.

pub mod bmp;
pub mod dymo_pdf;
pub mod error;
pub mod lbx;
pub mod odt;
pub mod qr;
pub mod spool;
pub mod tspl;
pub mod zip_store;

pub use crate::bmp::encode_monochrome_bmp_192;
pub use crate::dymo_pdf::compile_dymo_pdf;
pub use crate::error::RenderError;
pub use crate::lbx::compile_lbx_label;
pub use crate::odt::{compile_odt_label, escape_xml};
pub use crate::qr::QrMatrix;
pub use crate::spool::SpoolManager;
pub use crate::tspl::{compile_bt320_tspl, BT320_HEIGHT_DOTS, BT320_WIDTH_DOTS};
pub use crate::zip_store::ZipStoreBuilder;
