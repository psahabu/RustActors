use super::Actor;
use actor_agent::Agent;
use actor_context::Context;
use cage_message::CageMessage;

pub struct Stage {
	root: Context
}

impl Stage {
	pub fn new() -> Stage {
		// Create a channel for an Agent and an "Actor".
		let (send, recv) = channel::<CageMessage>();
		
		// TODO: finish impl
		// This "Actor" will handle "Find" requests, but will Send 
		// String Failures otherwise.
		/*
		spawn(proc() {
			loop {
				match recv.recv() {
						
		}
		*/

		// Setup an Agent and a dummy parent.
		let (_send, _recv) = channel::<CageMessage>();
		let dummy_parent = Agent::new(_send);

		// Return a Stage.
		Stage { root: Context::new(send, dummy_parent) }
	}

	/*
	 * Starting an Actor.
	 */
	fn start<T: Actor>(&mut self) -> Agent {
		self.root.start_child::<T>()
	}
}

