//! Hardware print transport abstractions and CUPS (`lpr`) dispatcher
//! for Label-Agent.

pub mod cups;
pub mod error;
pub mod transport;
pub mod usb_lp;

pub use crate::cups::CupsTransport;
pub use crate::error::PrintError;
pub use crate::transport::{DryRunTransport, PrintJob, PrintReceipt, PrintTransport};
pub use crate::usb_lp::{UsbLpTransport, DEFAULT_BT320_DEVICE, FALLBACK_USB_LP_DEVICE};
