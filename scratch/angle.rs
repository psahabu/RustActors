use std::os;
use std::from_str;
use std::f64;
use std::num::atan;

fn main() {
	let args = os::args();
	if args.len() == 3 {
		let x = get_scalar(args[1].clone());
		let y = get_scalar(args[2].clone());
		println!("{}pi radians", angle((x,y)));
	} else {
		println!("error: {} needs two real arguments", args[0]);
	}
}

fn get_scalar(arg: ~str) -> f64 {
	match from_str::from_str(arg) {
		Some(x) => x,
		None => 0.0
	}
}

fn angle(vector: (f64, f64)) -> f64 {
	let pi = f64::consts::PI;
	match vector {
		(0.0, y) if y < 0.0 => 1.5,
		(0.0, _) => 0.5,
		(x, y) => atan(y/x)/pi
	}
}

