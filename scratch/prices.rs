fn main() {
	let item = "pie";
	let price =
		match item {
			"fish" => 1.00,
			"pie"  => 3.14,
			_      => 0.00
		};
	println!("one {} costs ${}", item, price);
}
