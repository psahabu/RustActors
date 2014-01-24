fn main() {
	let mynum = 0;
	let myfun =
		match mynum {
			0 => { println!("zero") }
			_ => { println!("else") }
		};
	myfun
}
