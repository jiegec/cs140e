// FIXME: Make me compile! Diff budget: 2 lines.

// Do not change this definition.
enum MyEnum {
    A(String),
    B(String)
}

fn matcher(val: &MyEnum) -> &str {
    match *val {
        MyEnum::A(ref string) => string.as_str(),
        MyEnum::B(ref string) => string.as_str()
    }
}

fn main() { }
