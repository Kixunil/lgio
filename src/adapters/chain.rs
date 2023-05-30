use crate::BufRead;

/// Concatenates bytes from two readers - returned from [`BufRead::chain`].
pub struct Chain<L, R> {
    left: L,
    right: R,
    is_right: bool,
}

impl<L: BufRead, R: BufRead<ReadError=L::ReadError>> Chain<L, R> {
    pub(crate) fn new(left: L, right: R) -> Self {
        Chain {
            left,
            right,
            is_right: false,
        }
    }
}

impl<L: BufRead, R: BufRead<ReadError=L::ReadError>> BufRead for Chain<L, R> {
    type ReadError = L::ReadError;

    fn fill_buf(&mut self) -> Result<&[u8], Self::ReadError> {
        if self.is_right {
            self.right.fill_buf()
        } else {
            let mut buf = self.left.fill_buf()?;
            if buf.is_empty() {
                self.is_right = true;
                buf = self.right.fill_buf()?;
            }
            Ok(buf)
        }
    }

    fn consume(&mut self, amount: usize) {
        if self.is_right {
            self.right.consume(amount)
        } else {
            self.left.consume(amount)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::BufRead;

    #[test]
    fn chain_empty() {
        let mut reader = (&[]).chain(&[] as &[_]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
    }

    #[test]
    fn chain_non_empty_empty() {
        let mut reader = (&[42]).chain(&[] as &[_]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[42]);
        reader.consume(1);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
    }

    #[test]
    fn chain_empty_non_empty() {
        let mut reader = (&[]).chain(&[42u8] as &[_]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[42]);
        reader.consume(1);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
    }

    #[test]
    fn chain_empty_non() {
        let mut reader = (&[42]).chain(&[21u8] as &[_]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[42]);
        reader.consume(1);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[21]);
        reader.consume(1);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
        assert_eq!(reader.fill_buf().unwrap_or_else(|infallible| match infallible {}), &[]);
    }
}
