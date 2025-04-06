pub mod data;
pub mod de;
pub mod err;
pub mod ser;

pub use err::Error;
pub type Result<T, E = Error> = core::result::Result<T, E>;
