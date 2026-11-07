//! Hardware print transport abstractions and CUPS (`lpr`) dispatcher
//! for Label-Agent.

pub mod cups;
pub mod error;
pub mod transport;

pub use crate::cups::CupsTransport;
pub use crate::error::PrintError;
pub use crate::transport::{DryRunTransport, PrintJob, PrintReceipt, PrintTransport};
