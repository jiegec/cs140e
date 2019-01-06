// FIXME: Make me compile! Diff budget: 2 lines.
use std::io;

struct ReadWrapper<T: io::Read> {
    inner: T
}

impl<T: io::Read> io::Read for ReadWrapper<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.inner.read(buf)
    }
}

fn main() { }
