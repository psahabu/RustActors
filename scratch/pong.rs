extern mod extra;
use std::rand;
use std::rand::Rng;
use std::comm::SharedChan;
use std::clone::Clone;

struct Paddle {
	name: ~str
}

impl Clone for Paddle {
	fn clone(&self) -> Paddle {
		Paddle {name: self.name.clone()}
	}
}

fn pong(paddles: ~[Paddle]) {
	let mut sources = ~[];
	let mut drains = ~[];
	let mut count = 0;
	for _ in paddles.iter() {
		let (drain, source): (Port<()>, SharedChan<()>) = SharedChan::new();
		sources.push(source);
		drains.push(drain);
		count += 1;
	}

	for d in drains.move_iter() {
		let s = sources.clone();
		let p = paddles.clone();
		let num = count;
		spawn(proc() {
			d.recv();
			let mut rng = rand::rng();
			let to = rng.gen::<uint>() % num;
			if to >= num {
				println!("game over");
			} else {
				println!("{} has the puck", p[to].name);
				s[to].send(());
			}
		});
	}

	// start game
	sources[0].send(());
}

fn main() {
	let paddles = ~[
		Paddle {name: ~"Albert"},
		Paddle {name: ~"Bertha"},
		Paddle {name: ~"Carol"}
	];
	pong(paddles);
}
