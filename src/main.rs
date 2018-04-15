#[macro_use]
extern crate enumease_derive;
use std::slice::Iter;
use std::marker;


pub trait EnumIter {
    fn iterator() -> Iter<'static, Self> where Self: marker::Sized;
}

pub trait EnumFromStr: EnumIter {
    fn from_str(key: &str) -> Option<Self> where Self: marker::Sized;
}

#[derive(Debug,Clone,EnumDisplay, EnumIter, EnumFromStr)]
enum Foo {
    Bla,
    Faa,
    Caa,
}

fn main() {
    let f = Foo::Bla;
    println!("{}",f);
    println!("iteratin");
    for x in Foo::iterator() {
        println!("{}",x);
    }
    println!("{:?}",Foo::from_str("Bla"));
}