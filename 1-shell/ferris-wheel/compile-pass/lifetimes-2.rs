// FIXME: Make me compile! Diff budget: 3 lines.

// Do not modify the inner type &'a T.
struct RefWrapper<'a, T: 'a>(&'a T);

impl<'a, T> RefWrapper<'a, T> {
    fn inner(&self) -> &'a T {
        self.0
    }
}

// Do not modify this function.
pub fn main() {
    let x = 1;
    let mut r = &x;
    r = RefWrapper(r).inner();
}
