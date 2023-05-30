use crate::{BufRead, BufWrite};

/// Converts reader errors using closure `F` - returned from [`BufRead::map_read_err`].
pub struct MapReadErr<R, F> {
    reader: R,
    mapper: F,
}

impl<E, R: BufRead, F: FnMut(R::ReadError) -> E> MapReadErr<R, F> {
    pub(crate) fn new(reader: R, mapper: F) -> Self {
        MapReadErr {
            reader,
            mapper,
        }
    }
}

impl<E, R: BufRead, F: FnMut(R::ReadError) -> E> BufRead for MapReadErr<R, F> {
    type ReadError = E;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        self.reader.fill_buf().map_err(&mut self.mapper)
    }

    fn consume(&mut self, amount: usize) {
        self.reader.consume(amount)
    }
}

/// Converts writer errors using closure `F` - returned from [`BufWrite::map_write_err`].
pub struct MapWriteErr<W, F> {
    writer: W,
    mapper: F,
}

impl<E, W: BufWrite, F: FnMut(W::WriteError) -> E> MapWriteErr<W, F> {
    pub(crate) fn new(writer: W, mapper: F) -> Self {
        MapWriteErr {
            writer,
            mapper,
        }
    }
}

impl<E, W: BufWrite, F: FnMut(W::WriteError) -> E> BufWrite for MapWriteErr<W, F> {
    type WriteError = E;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        self.writer.write_all(bytes).map_err(&mut self.mapper)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        self.writer.flush().map_err(&mut self.mapper)
    }
}

/// Converts IO errors using closure `F` - returned from [`BufRead::map_err`].
pub struct MapErr<Io, F> {
    io: Io,
    mapper: F,
}

impl<E, Io: BufRead + BufWrite<WriteError=<Io as BufRead>::ReadError>, F: FnMut(Io::ReadError) -> E> MapErr<Io, F> {
    pub(crate) fn new(io: Io, mapper: F) -> Self {
        MapErr {
            io,
            mapper,
        }
    }
}

impl<E, R: BufRead, F: FnMut(R::ReadError) -> E> BufRead for MapErr<R, F> {
    type ReadError = E;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        self.io.fill_buf().map_err(&mut self.mapper)
    }

    fn consume(&mut self, amount: usize) {
        self.io.consume(amount)
    }
}

impl<E, W: BufWrite, F: FnMut(W::WriteError) -> E> BufWrite for MapErr<W, F> {
    type WriteError = E;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        self.io.write_all(bytes).map_err(&mut self.mapper)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        self.io.flush().map_err(&mut self.mapper)
    }
}

/// Converts IO errors using [`Into::into`] - returned from [`BufRead::unify_err`].
pub struct UnifyErr<Io, E> {
    io: Io,
    _phantom: core::marker::PhantomData<fn() -> E>,
}

impl<Io, E> UnifyErr<Io, E> where Io: BufRead + BufWrite, Io::ReadError: Into<E>, Io::WriteError: Into<E> {
    pub(crate) fn new(io: Io) -> Self {
        UnifyErr {
            io,
            _phantom: Default::default(),
        }
    }
}

impl<Io, E> BufRead for UnifyErr<Io, E> where Io: BufRead + BufWrite, Io::ReadError: Into<E>, Io::WriteError: Into<E> {
    type ReadError = E;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        self.io.fill_buf().map_err(Into::into)
    }

    fn consume(&mut self, amount: usize) {
        self.io.consume(amount)
    }
}

impl<Io, E> BufWrite for UnifyErr<Io, E> where Io: BufRead + BufWrite, Io::ReadError: Into<E>, Io::WriteError: Into<E> {
    type WriteError = E;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::WriteError> {
        self.io.write_all(bytes).map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        self.io.flush().map_err(Into::into)
    }
}
