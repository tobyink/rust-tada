mod tadaitem;
use crate::tadaitem::*;

fn main() {
	let i = TadaItem::parse("x (A) 2000-01-01 Foo bar @baz");
	println!("{:?}", i);
	println!("{}", i);
}
