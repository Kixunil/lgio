//! Implementations of various adapters.
//!
//! This module mostly contains types returned from trait methods. Thus you should not need to
//! worry about it much - just read the documentation of those methods.

mod take;
mod chain;
mod map_err;
#[cfg(feature = "std")]
mod std;

pub use take::*;
pub use chain::*;
pub use map_err::*;
#[cfg(feature = "std")]
pub use self::std::*;
