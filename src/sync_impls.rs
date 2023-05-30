use crate::{BufRead, BufWrite, Empty, Sink, Null};
use crate::error::BufferOverflow;

impl<T: BufRead + ?Sized> BufRead for &'_ mut T {
    type ReadError = T::ReadError;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        (*self).fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        (*self).consume(amount)
    }
}

impl<'a> BufRead for &'a [u8] {
    type ReadError = core::convert::Infallible;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        Ok(*self)
    }

    fn consume(&mut self, amount: usize) {
        *self = &self[amount..];
    }
}

impl<'a> BufRead for &'a mut [u8] {
    type ReadError = core::convert::Infallible;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        Ok(*self)
    }

    fn consume(&mut self, amount: usize) {
        let this = core::mem::replace(self, &mut []);
        *self = &mut this[amount..];
    }
}

impl<T: BufWrite + ?Sized> BufWrite for &'_ mut T {
    type WriteError = T::WriteError;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        (*self).write_all(bytes)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        (*self).flush()
    }
}

impl<'a> BufWrite for &'a mut [u8] {
    type WriteError = BufferOverflow;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        if bytes.len() > self.len() {
            return Err(BufferOverflow::new(bytes.len() - self.len()));
        }

        let this = core::mem::replace(self, &mut []);
        let (target, remaining) = this.split_at_mut(bytes.len());
        target.copy_from_slice(bytes);
        *self = remaining;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        Ok(())
    }
}

impl BufRead for Empty {
    type ReadError = core::convert::Infallible;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        Ok(&[])
    }

    fn consume(&mut self, amount: usize) {
        debug_assert_eq!(amount, 0);
    }
}

impl BufWrite for Sink {
    type WriteError = core::convert::Infallible;

    fn write_all(&mut self, _bytes: &[u8]) -> Result<(), Self::WriteError> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        Ok(())
    }
}

impl BufRead for Null {
    type ReadError = core::convert::Infallible;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        Ok(&[])
    }

    fn consume(&mut self, amount: usize) {
        debug_assert_eq!(amount, 0);
    }
}

impl BufWrite for Null {
    type WriteError = core::convert::Infallible;

    fn write_all(&mut self, _bytes: &[u8]) -> Result<(), Self::WriteError> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T: BufRead + ?Sized> BufRead for alloc::boxed::Box<T> {
    type ReadError = T::ReadError;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        (**self).fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        (**self).consume(amount)
    }
}

#[cfg(feature = "alloc")]
impl<T: BufWrite + ?Sized> BufWrite for alloc::boxed::Box<T> {
    type WriteError = T::WriteError;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        (**self).write_all(bytes)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        (**self).flush()
    }
}

#[cfg(feature = "alloc")]
impl BufWrite for alloc::vec::Vec<u8> {
    type WriteError = core::convert::Infallible;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        self.extend_from_slice(bytes);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T: std::io::Read> BufRead for std::io::BufReader<T> {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        fill_buf(self)
    }

    fn consume(&mut self, amount: usize) {
        std::io::BufRead::consume(self, amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        std::io::Read::read_to_end(self, buf)
    }
}

#[cfg(feature = "std")]
impl<T: AsRef<[u8]>> BufRead for std::io::Cursor<T> {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        fill_buf(self)
    }

    fn consume(&mut self, amount: usize) {
        std::io::BufRead::consume(self, amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        std::io::Read::read_to_end(self, buf)
    }
}

#[cfg(feature = "std")]
impl<T: std::io::BufRead, U: std::io::BufRead> BufRead for std::io::Chain<T, U> {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        fill_buf(self)
    }

    fn consume(&mut self, amount: usize) {
        std::io::BufRead::consume(self, amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        std::io::Read::read_to_end(self, buf)
    }
}

#[cfg(feature = "std")]
impl<T: std::io::BufRead> BufRead for std::io::Take<T> {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        fill_buf(self)
    }

    fn consume(&mut self, amount: usize) {
        std::io::BufRead::consume(self, amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        std::io::Read::read_to_end(self, buf)
    }
}

#[cfg(feature = "std")]
impl BufRead for std::io::Empty {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        fill_buf(self)
    }

    fn consume(&mut self, amount: usize) {
        std::io::BufRead::consume(self, amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        std::io::Read::read_to_end(self, buf)
    }
}

#[cfg(feature = "std")]
impl BufRead for std::io::StdinLock<'_> {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        fill_buf(self)
    }

    fn consume(&mut self, amount: usize) {
        std::io::BufRead::consume(self, amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        std::io::Read::read_to_end(self, buf)
    }
}

#[cfg(feature = "std")]
impl<T: std::io::Write> BufWrite for std::io::BufWriter<T> {
    type WriteError = std::io::Error;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        std::io::Write::write_all(self, bytes)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        std::io::Write::flush(self)
    }
}

#[cfg(feature = "std")]
impl BufWrite for std::io::Sink {
    type WriteError = std::io::Error;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        std::io::Write::write_all(self, bytes)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        std::io::Write::flush(self)
    }
}

#[cfg(feature = "std")]
impl BufWrite for std::io::StdoutLock<'_> {
    type WriteError = std::io::Error;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        std::io::Write::write_all(self, bytes)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        std::io::Write::flush(self)
    }
}

#[cfg(feature = "std")]
impl BufWrite for std::io::StderrLock<'_> {
    type WriteError = std::io::Error;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        std::io::Write::write_all(self, bytes)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        std::io::Write::flush(self)
    }
}

#[cfg(feature = "std")]
fn fill_buf<'a, R: std::io::BufRead>(reader: &'a mut R) -> std::io::Result<&'a [u8]> {
    loop {
        match std::io::BufRead::fill_buf(reader) {
            // SAFETY: this works around a borrowchecker bug
            // See https://github.com/rust-lang/rust/issues/51132
            Ok(bytes) => return Ok(unsafe { &*(bytes as *const _) }),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => (),
            Err(error) => return Err(error),
        }
    }
}
