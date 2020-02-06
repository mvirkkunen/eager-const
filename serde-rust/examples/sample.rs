use serde_rust;
use serde::Serialize;

#[derive(Serialize)]
struct Foo {
    name: String,
    age: u32,
}

fn main() {
    println!("{}", serde_rust::to_string(&Foo { name: "Kalle".into(), age: 42 }).unwrap());
    println!("{}", serde_rust::to_string(&vec![1, 2, 3, 4]).unwrap());
}