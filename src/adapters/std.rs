use crate::{BufRead, BufWrite};
use std::io;

/// Provides [`std::io`] traits for applicable [`BufRead`] and [`BufWrite`] implementors - returned
/// from [`BufRead::into_std`].
pub struct AsStd<Io>(super::UnifyErr<Io, io::Error>);

impl<Io: BufRead + BufWrite> AsStd<Io> where Io::ReadError: Into<io::Error>, Io::WriteError: Into<io::Error> {
    pub(crate) fn new(io: Io) -> Self {
        AsStd(io.unify_err())
    }
}

impl<Io: BufRead + BufWrite> io::Read for AsStd<Io> where Io::ReadError: Into<io::Error>, Io::WriteError: Into<io::Error> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.0.fill_buf()?;
        let to_copy = buf.len().min(read.len());
        buf[..to_copy].copy_from_slice(&read[..to_copy]);
        self.0.consume(to_copy);
        Ok(to_copy)
    }
}

impl<Io: BufRead + BufWrite> io::BufRead for AsStd<Io> where Io::ReadError: Into<io::Error>, Io::WriteError: Into<io::Error> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.0.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.0.consume(amount)
    }
}

impl<Io: BufRead + BufWrite> io::Write for AsStd<Io> where Io::ReadError: Into<io::Error>, Io::WriteError: Into<io::Error> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write_all(buf)?;
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// Provides [`std::io`] read traits for applicable [`BufRead`] implementors - returned from
/// [`BufRead::into_std`].
pub struct AsStdReader<Io>(Io);

impl<Io: BufRead> AsStdReader<Io> where Io::ReadError: Into<io::Error> {
    pub(crate) fn new(io: Io) -> Self {
        AsStdReader(io)
    }
}

impl<Io: BufRead> io::Read for AsStdReader<Io> where Io::ReadError: Into<io::Error> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.0.fill_buf().map_err(Into::into)?;
        let to_copy = buf.len().min(read.len());
        buf[..to_copy].copy_from_slice(&read[..to_copy]);
        self.0.consume(to_copy);
        Ok(to_copy)
    }
}

impl<Io: BufRead> io::BufRead for AsStdReader<Io> where Io::ReadError: Into<io::Error> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.0.fill_buf().map_err(Into::into)
    }

    fn consume(&mut self, amount: usize) {
        self.0.consume(amount)
    }
}

/// Provides [`std::io::Write`] for applicable [`BufWrite`] implementors - returned from
/// [`BufRead::into_std`].
pub struct AsStdWriter<Io>(Io);

impl<Io: BufWrite> io::Write for AsStdWriter<Io> where Io::WriteError: Into<io::Error> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write_all(buf).map_err(Into::into)?;
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf).map_err(Into::into)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush().map_err(Into::into)
    }
}

/// Provides [`BufRead`] implementation for [`std::io::BufRead`] implementors - returned
/// from [`crate::from_std_reader`].
pub struct StdBufRead<Io>(Io);

impl<Io: io::BufRead> StdBufRead<Io> {
    pub(crate) fn new(io: Io) -> Self {
        StdBufRead(io)
    }
}

impl<Io: io::BufRead> BufRead for StdBufRead<Io> {
    type ReadError = std::io::Error;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        loop {
            match self.0.fill_buf() {
                // SAFETY: this works around a borrowchecker bug
                // See https://github.com/rust-lang/rust/issues/51132
                Ok(bytes) => break Ok(unsafe { &*(bytes as *const _) }),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => (),
                Err(error) => break Err(error),
            }
        }
    }

    fn consume(&mut self, amount: usize) {
        self.0.consume(amount)
    }

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, Self::ReadError> {
        self.0.read_to_end(buf)
    }
}
