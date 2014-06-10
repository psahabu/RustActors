mod bushel {
	struct Candle;
	impl Candle {
		fn hello_candle(&self) {
			println!("Fire!");
		}
	}
}

fn main() {
	let candle = Candle;
	candle.hello_candle();
}
