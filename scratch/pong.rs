extern mod extra;
use std::rand;
use std::rand::Rng;
use std::comm::SharedChan;
use std::clone::Clone;

// rustc can derive Clone instances for you
#[deriving(Clone)]
struct Paddle {
	name: ~str
}

fn pong(paddles: ~[Paddle]) {
	let mut sources = ~[];
	let mut drains = ~[];
	// instead of manually counting, use .len()
	for _ in paddles.iter() {
		let (drain, source): (Port<bool>, SharedChan<bool>) = SharedChan::new();
		sources.push(source);
		drains.push(drain);
	}
	// freeze the vectors now that they're built. the the compiler will stop us accidently mutating them.
	let sources = sources;
	let drains = drains;
	let num = paddles.len(); // just do this once. it'll be copied by the spawned closures

	for d in drains.move_iter() {
		let s = sources.clone();
		let p = paddles.clone();
		spawn(proc() {
			// just create one generator per task rather than one per loop iteration
			let mut rng = rand::rng();
			// why not use a while loop instead of loop + if/else break?
			while d.recv() {
				let mut rng = rand::rng();
				// rather than modulo, try gen_range
				let to = rng.gen_range(0u, num+1);
				if to == num {
					println!("game over");
					for source in s.iter() {
						source.send(false);
					}
					break;
				} else {
					println!("{} has the puck", p[to].name);
					s[to].send(true);
				}

			}
		});
	}

	// start game
	sources[0].send(true);
}

fn main() {
	let paddles = ~[
		Paddle {name: ~"Albert"},
		Paddle {name: ~"Bertha"},
		Paddle {name: ~"Carol"}
	];
	pong(paddles);
}
