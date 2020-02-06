use eager_const::eager_const;

eager_const! {
    const FOO: &str = {
        let mut s = String::new();

        s.push_str("Hello! Generating things at compile time is as easy as ");

        for n in 1..=3 {
            push_number(&mut s, n);
        }

        s.push_str("!");
        s.push_str(" Maybe some day we will be able to use a plain const fn for this.");

        s
    };
}

fn push_number(s: &mut String, num: usize) {
    match num {
        1 => s.push_str("one, "),
        2 => s.push_str("two, "),
        n => s.push_str(&n.to_string()),
    };
}

fn main() {
    const REALLY_CONST: &str = FOO; // it's really const

    println!("{}", REALLY_CONST);
}