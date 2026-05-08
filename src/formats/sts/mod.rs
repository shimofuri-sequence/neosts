mod read;
mod write;

pub use read::{STS_MAGIC, StsCel, StsError, StsFile};
pub use write::{sheet_to_sts, write_sheet_to_path};
