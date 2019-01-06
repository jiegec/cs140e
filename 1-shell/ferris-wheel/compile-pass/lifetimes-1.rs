// FIXME: Make me compile! Diff budget: 1 line.

struct StrWrapper<'a>(&'a str);

impl<'a> StrWrapper<'a> {
    fn inner(&self) -> &'a str {
        self.0
    }
}

// Do not modify this function.
pub fn main() {
    let string = "Hello!";
    let wrapper = StrWrapper(&string);
    let _: &'static str = wrapper.inner();
}
