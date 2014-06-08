use std::rand;
use std::rand::Rng;
use std::clone::Clone;
use std::vec::Vec;
use std::string::String;

// rustc can derive Clone instances for you
#[deriving(Clone)]
struct Paddle {
	name: String 
}

fn pong(paddles: ~[Paddle]) {
	let mut sources = Vec::new();
	let mut drains = Vec::new();
	// instead of manually counting, use .len()
	for _ in paddles.iter() {
		let (send,recv) = channel::<bool>();
		sources.push(send);
		drains.push(recv);
	}
	// freeze the vectors now that they're built. the the compiler will stop us accidently mutating them.
	let sources = sources;
	let drains = drains;
	let num = paddles.len(); // just do this once. it'll be copied by the spawned closures

	for d in drains.move_iter() {
		// capture variables
		let s = sources.clone();
		let p = paddles.clone();
		spawn(proc() {
			// just create one generator per task rather than one per loop iteration
			let mut rng = rand::task_rng();
			// why not use a while loop instead of loop + if/else break?
			while d.recv() {
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
					s.get(to).send(true);
				}
			}
		});
	}

	// start game
	sources.get(0).send(true);
}

fn main() {
	let paddles = box [
		Paddle {name: "Albert".to_string()},
		Paddle {name: "Bertha".to_string()},
		Paddle {name: "Carol".to_string()}
	];
	pong(paddles);
}
