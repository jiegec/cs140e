// FIXME: Make me pass! Diff budget: 2 lines.

#[derive(Debug, PartialEq)]
struct MyType(usize);

pub fn main() {
    let mut x = MyType(1);
    {
        let y = &x;
        assert_eq!(*y, MyType(1));
    }

    // Do not modify this line.
    let x = &mut x;
}
