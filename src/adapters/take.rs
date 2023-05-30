use core::convert::TryFrom;
use crate::BufRead;

/// Provides a limited number of bytes from underlying reader - returned from [`BufRead::take`].
pub struct Take<R> {
    reader: R,
    limit: u64,
    #[cfg(debug_assertions)]
    last_len: usize,
}

impl<R: BufRead> Take<R> {
    pub(crate) fn new(reader: R, limit: u64) -> Self {
        Take {
            reader,
            limit,
            #[cfg(debug_assertions)]
            last_len: 0,
        }
    }
}

impl<R: BufRead> BufRead for Take<R> {
    type ReadError = R::ReadError;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        match self.reader.fill_buf() {
            Ok(buf) => {
                #[cfg(debug_assertions)]
                {
                    self.last_len = buf.len();
                }
                Ok(&buf[..min(self.limit, buf.len())])
            },
            Err(error) => Err(error),
        }
    }

    fn consume(&mut self, amount: usize) {
        #[cfg(debug_assertions)]
        assert!(amount <= self.last_len);
        // if amount is within bounds this won't overflow because of how len was computed above
        self.limit -= amount as u64;

        self.reader.consume(amount);
    }
}

fn min(a: u64, b: usize) -> usize {
    match usize::try_from(a) {
        Ok(a) => a.min(b),
        Err(_) => b,
    }
}

#[cfg(test)]
mod tests {
    use crate::BufRead;

    #[test]
    fn take_zero() {
        let mut reader = (&[1, 2, 3]).take(0);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        let mut buf = [0; 1];

        assert!(reader.read_exact(&mut buf).is_err());
    }

    #[test]
    fn take_one() {
        let mut reader = (&[1, 2, 3]).take(1);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[1]);
        reader.consume(1);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        let mut buf = [0; 1];
        assert!(reader.read_exact(&mut buf).is_err());
    }
}
