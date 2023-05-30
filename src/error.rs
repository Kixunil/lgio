//! A collection of error types related to IO.

use core::fmt;

/// Error returned when a buffer is full.
#[derive(Debug, Clone)]
pub struct BufferOverflow {
    bytes_past_end: usize,
}

impl BufferOverflow {
    /// Constructs the error.
    pub fn new(bytes_past_end: usize) -> Self {
        BufferOverflow {
            bytes_past_end,
        }
    }
}

impl fmt::Display for BufferOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "attempted to write {} bytes past the end of the buffer", self.bytes_past_end)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BufferOverflow {}

/// Error returned when more bytes are required from a reader but no more are available.
#[derive(Debug, Clone)]
pub struct UnexpectedEnd {
    total_required: usize,
    available: usize,
}

impl UnexpectedEnd {
    /// Constructs the error.
    pub fn new(total_required: usize, available: usize) -> Self {
        UnexpectedEnd {
            total_required,
            available,
        }
    }
}

impl fmt::Display for UnexpectedEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} bytes were required but only {} bytes were read", self.total_required, self.available)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnexpectedEnd {}

/// Error returned from [`BufRead::read_exact`](crate::BufRead::read_exact).
#[derive(Debug, Clone)]
pub enum ReadExactError<E> {
    /// There are not enough bytes.
    UnexpectedEnd(UnexpectedEnd),
    /// Reading failed.
    ReadingFailed(E),
}

impl<E> ReadExactError<E> {
    /// Shorthand for `UnexpectedEnd(UnexpectedEnd::new())`.
    pub fn unexpected_end(total_required: usize, available: usize) -> Self {
        UnexpectedEnd::new(total_required, available).into()
    }

    /// Transforms `ReadingFailed` variant using the closure `f`.
    pub fn map_read_err<E2, F: FnOnce(E) -> E2>(self, f: F) -> ReadExactError<E2> {
        match self {
            ReadExactError::UnexpectedEnd(error) => ReadExactError::UnexpectedEnd(error),
            ReadExactError::ReadingFailed(error) => ReadExactError::ReadingFailed(f(error)),
        }
    }
}

impl ReadExactError<core::convert::Infallible> {
    /// Statically proves that `UnexpectedEnd` is the only possible error and converts to it.
    pub fn into_unexpected_end(self) -> UnexpectedEnd {
        match self {
            ReadExactError::UnexpectedEnd(error) => error,
            ReadExactError::ReadingFailed(never) => match never {},
        }
    }
}

impl<E> From<UnexpectedEnd> for ReadExactError<E> {
    fn from(error: UnexpectedEnd) -> Self {
        ReadExactError::UnexpectedEnd(error)
    }
}


impl<E> fmt::Display for ReadExactError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadExactError::UnexpectedEnd(_) => write!(f, "unexpected end"),
            ReadExactError::ReadingFailed(_) => write!(f, "reading failed"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for ReadExactError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadExactError::UnexpectedEnd(error) => Some(error),
            ReadExactError::ReadingFailed(error) => Some(error),
        }
    }
}
