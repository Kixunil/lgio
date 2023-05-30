//! # Lightweight generic IO
//!
//! Less error-prone, `no_std` IO traits generic over their error types.
//!
//! This is a simplified version of [`genio`](https://docs.rs/genio) containing only buffered
//! traits.
//!
//! ## Advantages over `std`
//!
//! * `no_std` - doesn't require an operating system
//! * Error being associated type is more flexible
//! * Less error-prone - no `read` and `write` methods which are often mistaken for `read_all` or
//!   `write_all`
//!
//! ## Advantages over `genio`
//!
//! * Simpler
//! * Less `unsafe` to deal with uninitialized bytes (currently none, may change in the future)
//! * Most uses of IO need some buffering anyway
//! * Less error-prone - no `read` and `write` methods which are often mistaken for `read_all` or
//!   `write_all`
//! * No `FlushError` makes error handling simpler
//!
//! ## Target audience
//!
//! Mainly serialization libraries and their consumers.
//! Can be also usful for simple protocols that don't need precise control of `read` and `write`
//! calls.
//!
//! Probably should *not* be used in lower layers.
//!
//! ## Usage overview
//!
//! The [`BufRead`] trait is very similar to the one from `std`. The biggest differences are error
//! type and lack of error-prone `read` method. Since it is implemented on `std::io::BufReader` and
//! primitive `std` types you can use it exactly the same as [`std::io::BufRead`] in most cases.
//! There's an added benefit that you can statically prove reading from `&[u8]` will not fail (but
//! it can return `UnexpectedEnd`).
//!
//! Similarly, [`BufWrite`] is just [`std::io::Write`] with error being associated and missing
//! `write` method. It still requires that writing is either buffered or fast because that's what
//! most encoders need.
//!
//! ## Features
//!
//! * `std` - integration with the standard library: implementations and adapters
//! * `alloc` - additional features requiring allocation
//!
//! ## MSRV
//!
//! The crate intends to have conservative MSRV and only bump it when it provides significant
//! benefit and at most to the version available in latest Debian stable. Currently tested MSRV is
//! 1.41.1 (Debian oldstable) but due to its simplicity it's possible it works on even lower
//! versions.
//!
//! Some features may be only available in newer Rust versions. Thus it is recommended to use
//! recent Rust if possible.

#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod adapters;
pub mod error;
mod sync_impls;

use adapters::*;
use error::*;

/// A `BufRead` is a reader which has an internal buffer, allowing it to perform reading
/// efficiently.
///
/// This trait has interface similar to [`std::io::BufRead`] with these notable differences:
/// * Doesn't require `Read`-like trait
/// * Error type is associated
/// * It provides some methods that are on [`std::io::Read`]
/// * It doesn't provide the error-prone [`read`](std::io::Read::read) method
pub trait BufRead {
    /// The error returned when reading fails.
    ///
    /// This is commonly [`std::io::Error`] but some interesting types such as `&[u8]` never fail.
    type ReadError;

    /// Returns the contents of the internal buffer, filling it with more data from the underlying
    /// source if the buffer is empty.

    /// This function is a lower-level call. It needs to be paired with the [`consume`] method to
    /// function properly. When calling this method, none of the contents will be “read” in the
    /// sense that later calling `read_exact` may return the same contents. As such, [`consume`]
    /// must be called with the number of bytes that are consumed from this buffer to ensure that
    /// the bytes are never returned twice.

    /// An empty buffer returned indicates that the stream has reached end (EOF).
    ///
    /// Note that implementors should handle errors that correspond to
    /// [`std::io::ErrorKind::Interrupted`] and restart the operation. All `std` adapters in this
    /// library do so.
    ///
    /// # Errors
    ///
    /// This function will return an I/O error if the underlying reader was read, but returned an
    /// error.
    ///
    /// [`consume`]: Self::consume
    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError>;

    /// Tells this buffer that `amount` bytes have been consumed from the buffer, so they 
    /// should no longer be returned in calls to `fill_buf`, `read_exact` and others.
    ///
    /// This function is a lower-level call. It needs to be paired with the [`fill_buf`] method to
    /// function properly. This function does not perform any I/O, it simply informs this object
    /// that some amount of its buffer, returned from [`fill_buf`], has been consumed and should no
    /// longer be returned. As such, this function may do odd things if [`fill_buf`] isn't called
    /// before calling it.
    ///
    /// The `amt` must be `<=` the number of bytes in the buffer returned by [`fill_buf`]. Failing
    /// to uphold this requirement may lead to panics or other arbitrary bugs that are **not**
    /// Undefined Behavior.
    ///
    /// [`fill_buf`]: Self::fill_buf
    fn consume(&mut self, amount: usize);

    /// Reads a single byte from the reader.
    /// 
    /// # Errors
    ///
    /// * Returns `Err` if reading fails.
    /// * Returns `Ok(None)` if there are no more bytes.
    fn read_byte(&mut self) -> Result<Option<u8>, Self::ReadError> {
        Ok(self.fill_buf()?.first().copied().map(|byte| { self.consume(1); byte }))
    }

    /// Read the exact number of bytes required to fill `buf`.
    ///
    /// This function reads as many bytes as necessary to completely fill the specified buffer
    /// `buf`.
    ///
    /// No guarantees are provided about the contents of `buf` when this function is called, so
    /// implementations cannot rely on any property of the contents of `buf` being true. It is
    /// recommended that implementations only write data to `buf` instead of reading its contents.
    ///
    /// Note that this method doesn't allow skipping initialization of the buffer which may lead to
    /// decreased performance. Consider reading the bytes off the buffer returned by `fill_buf`
    /// instead.
    ///
    /// # Errors
    ///
    /// If this function encounters an "end of file" before completely filling
    /// the buffer, it returns [`ReadExactError::UnexpectedEnd`].  The contents of `buf` are
    /// unspecified in this case.
    ///
    /// If any other read error is encountered then this function immediately returns. The contents
    /// of `buf` are unspecified in this case.
    ///
    /// If this function returns an error, it is unspecified how many bytes it has read, but it
    /// will never consume more than would be necessary to completely fill the buffer.
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), ReadExactError<Self::ReadError>> {
        let required = buf.len();
        while !buf.is_empty() {
            let read = self.fill_buf().map_err(ReadExactError::ReadingFailed)?;
            if read.is_empty() {
                return Err(ReadExactError::unexpected_end(required, buf.len()));
            }
            let to_copy = buf.len().min(read.len());
            let (target, remaining) = buf.split_at_mut(to_copy);
            target.copy_from_slice(&read[..to_copy]);
            buf = remaining;
        }
        Ok(())
    }

    /// Read all bytes until EOF in this source, placing them into `buf`.
    ///
    /// All bytes read from this source will be appended to the specified buffer
    /// `buf`. This function will continuously call [`fill_buf`]/[`consume`] and append more data to
    /// `buf` until [`fill_buf`] returns either [`Ok(0)`] or an error.
    ///
    /// If successful, this function will return the total number of bytes read.
    ///
    /// # Errors
    ///
    /// If any read error is encountered then this function immediately returns. Any bytes 
    /// which have already been read will be appended to `buf`.
    /// 
    /// # Example
    ///
    /// ```
    /// use lgio::BufRead;
    ///
    /// # let data = [1, 2, 3];
    /// # let mut reader = &data as &[_];
    /// let mut buf = Vec::new();
    /// reader.read_to_end(&mut buf).unwrap_or_else(|error| match error {});
    /// # assert_eq!(buf, data);
    /// ```
    ///
    /// [`fill_buf`]: Self::fill_buf
    /// [`consume`]: Self::consume
    /// [`Ok(0)`]: Ok
    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        let mut total = 0;
        loop {
            let read = self.fill_buf()?;
            if read.is_empty() {
                break Ok(total);
            }
            buf.extend_from_slice(read);
            let len = read.len();
            total += len;
            self.consume(len);
        }
    }

    /// Creates an adapter which will read at most `limit` bytes from it.
    ///
    /// This function returns a new instance of `BufRead` which will read at most `limit` bytes,
    /// after which it will always return EOF ( [`Ok(&\[\])`] ). Any read errors will not count towards
    /// the number of bytes read and future calls to [`fill_buf`] may succeed.
    ///
    /// [`fill_buf`]: Self::fill_buf
    /// [`Ok(&\[\])`]: Result::Ok
    fn take(self, limit: u64) -> Take<Self> where Self: Sized {
        Take::new(self, limit)
    }

    /// Creates an adapter which will chain this stream with another.
    ///
    /// The returned `BufRead` instance will first read all bytes from this object until end (EOF)
    /// is encountered. Afterwards the output is equivalent to the output of `next`.
    fn chain<R: BufRead<ReadError=Self::ReadError>>(self, other: R) -> Chain<Self, R> where Self: Sized {
        Chain::new(self, other)
    }

    /// Returns an adapter converting read and write errors using the closure `f`.
    fn map_err<E, F>(self, f: F) -> MapErr<Self, F> where Self: BufWrite<WriteError=<Self as BufRead>::ReadError> + Sized, F: FnMut(Self::ReadError) -> E {
        MapErr::new(self, f)
    }

    /// Returns an adapter converting read errors using the closure `f`.
    fn map_read_err<E, F>(self, f: F) -> MapReadErr<Self, F> where Self: Sized, F: FnMut(Self::ReadError) -> E {
        MapReadErr::new(self, f)
    }

    /// Returns an adapter converting read and write errors using their [`Into::into`]
    /// implementation.
    fn unify_err<E>(self) -> UnifyErr<Self, E> where Self: BufWrite + Sized, Self::ReadError: Into<E>, Self::WriteError: Into<E> {
        UnifyErr::new(self)
    }

    /// Returns an adapter providing implementations of [`std::io::Read`], [`std::io::BufRead`],
    /// and [`std::io::Write`].
    #[cfg(feature = "std")]
    fn into_std(self) -> AsStd<Self> where Self: BufWrite + Sized, Self::ReadError: Into<std::io::Error>, Self::WriteError: Into<std::io::Error> {
        AsStd::new(self)
    }

    /// Returns an adapter providing implementations of [`std::io::Read`] and [`std::io::BufRead`].
    #[cfg(feature = "std")]
    fn into_std_reader(self) -> AsStdReader<Self> where Self: Sized, Self::ReadError: Into<std::io::Error> {
        AsStdReader::new(self)
    }

    /// Creates a "by reference" adapter for this instance of `BufRead`.
    ///
    /// The returned adapter also implements `BufRead` and will simply borrow this current writer.
    fn by_ref(&mut self) -> &mut Self {
        self
    }
}

/// A trait for objects which are buffered, byte-oriented sinks.
///
/// Implementors of the `BufWrite` trait are sometimes called 'writers'.
///
/// Writers are defined by two required methods, [`write_all`] and [`flush`]:
///
/// * The [`write_all`] method will attempt to write all of the given data into the object
/// * The [`flush`] method is useful for adapters and explicit buffers themselves for ensuring that
///   all buffered data has been pushed out to the 'true sink'.
///
/// Writers are intended to be composable with one another. Many implementors
/// throughout `lgio` take and provide types which implement the `BufWrite` trait.
///
/// Note that the implementations of `BufWrite` don't require that the type actually contains a
/// buffer. It's perfectly OK to not have it if writing doesn't involve context switch or similar
/// expensive operations. In other words, if performance of the writing to the writer is roughly
/// same when bytes are fed individually or in large chunks then the writer may implement
/// `BufRead`. If not it should provide some mechanism to add a buffer so it becomes less expensive
/// to write byte-by-byte.
///
/// [`write_all`]: BufWrite::write_all
/// [`flush`]: BufWrite::flush
pub trait BufWrite {
    /// The error returned when writing fails.
    ///
    /// This is commonly [`std::io::Error`] but some interesting types such as `Vec<u8>` never fail.
    type WriteError;


    /// Attempts to write an entire buffer into this writer.
    ///
    /// This method will not return until the entire buffer has been successfully written or an
    /// error occurs. The first error that is generated from this method will be returned.
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError>;

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    ///
    /// # Errors
    ///
    /// It is considered an error if not all bytes could be written due to
    /// I/O errors.
    fn flush(&mut self) -> Result<(), Self::WriteError>;

    /// Returns an adapter converting write errors using the closure `f`.
    fn map_write_err<E, F: FnMut(Self::WriteError) -> E>(self, f: F) -> MapWriteErr<Self, F> where Self: Sized {
        MapWriteErr::new(self, f)
    }

    /// Creates a "by reference" adapter for this instance of `BufWrite`.
    ///
    /// The returned adapter also implements `BufWrite` and will simply borrow this current writer.
    fn by_ref(&mut self) -> &mut Self {
        self
    }
}

/// Returns a reader that has no data (is at end).
pub fn empty() -> Empty {
    Empty
}

/// Returns a writer that discards all data written to it.
pub fn sink() -> Sink {
    Sink
}

/// Returns a reader-writer that returns no data and discards all data written to it.
///
/// This is analogous to opening `/dev/null` on Linux/Unix but is zero-cost.
pub fn null() -> Null {
    Null
}

/// A reader with no data (always at the end).
#[non_exhaustive]
pub struct Empty;

/// A writer which throws away (ignores) the data.
#[non_exhaustive]
pub struct Sink;

/// A reader-writer that has no data and throws away all data written to it.
///
/// This is analogous to `/dev/null` on Linux/Unix.
#[non_exhaustive]
pub struct Null;

/// Returns an adapter for arbitrary [`std::io::BufRead`]er.
///
/// This is only intended for types from external crates implementing `std::io::BufRead`.
/// Types from `std` that implement `std::io::BufRead` already implement `BufRead`.
///
/// Note that write version is **not** provided beause there's no `std::io::BufWrite`.
#[cfg(feature = "std")]
pub fn from_std_reader<R: std::io::BufRead>(reader: R) -> StdBufRead<R> {
    StdBufRead::new(reader)
}
