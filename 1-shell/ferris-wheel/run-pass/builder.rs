// FIXME: Make me pass! Diff budget: 30 lines.

struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    fn default() -> Builder {
        Builder {
            string: None, 
            number: None 
        }
    }

    fn string<T: Into<String>>(&mut self, s: T) -> &mut Self {
        self.string = Some(s.into())
        self
    }

    fn number(&mut self, n: usize) -> &mut Self {
        self.number = Some(n);
        self
    }
}

impl ToString for Builder {
    fn to_string(&self) -> String {
        [
            &self.string,
            &self.number.map(|x| x.to_string()),
        ].iter()
            .filter(|x| x.is_some())
            .map(|&x| x.clone().unwrap())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

// Do not modify this function.
fn main() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default().string("heap!".to_owned()).to_string();

    assert_eq!(c, "heap!");
}
