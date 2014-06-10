use sync::Arc;
use sync::Mutex;

use super::Actor;
use super::Message;
use actor_agent::Agent;
use actor_context::Context;
use cage_message::CageMessage;
	use cage_message::UserMessage;
	use cage_message::Find;
	use cage_message::Terminated;
	use cage_message::Failure;
	use cage_message::Undelivered;
	use cage_message::Watch;
	use cage_message::Unwatch;
	use cage_message::Kill;

pub struct Stage {
	root: Arc<Mutex<Context>>
}

impl Stage {
	/*
	 * Starting an Actor.
	 */
	pub fn start<T: Actor>(&mut self) -> Agent {
		self.root.lock().start_child::<T>()
	}
	
	// A context object for Actors to be created in.
	pub fn new() -> Stage {
		// Create a channel for an Agent.
		let (send, recv) = channel::<CageMessage>();
	
		// Setup an Agent and a dummy parent.
		let (_send, _recv) = channel::<CageMessage>();
		let dummy_parent = Agent::new(_send);

		// Create a context.
		let root_context = Context::root(send, dummy_parent);

		// Wrap the context in a lock.
		let root_context = Arc::new(Mutex::new(root_context));

		// Feed the wrapped context to an "Actor". 
		Stage::start_root(recv, root_context.clone());

		// Return a Stage.
		Stage { root: root_context }
	}

	// Starts an "Actor" that will handle "Find" requests, but
	// will Send String Failures otherwise.
	fn start_root(recv: Receiver<CageMessage>, context: Arc<Mutex<Context>>) {
		spawn(proc() {
			loop {
				match recv.recv() {
					UserMessage(msg, sender) =>
						sender.deliver(Stage::stage_failure("ERROR: Tried to deliver a message to Stage.", &context)),
					Find(path, msg, sender) => {
						let mut _path = path;
						match _path.pop() {
							Some(ref s) =>
								match s.as_slice() {
									"*" => {
										for child in context.lock().children().iter() {
											child.deliver(UserMessage(msg.clone_me(), sender.clone()));
										}	
									},
									".." =>	sender.deliver(Stage::stage_failure("ERROR: Tried to access parent of Stage.", &context)),
									_ => {
										for child in context.lock().children().iter() {
											if child.name() == *s {
												child.deliver(Find(_path, msg, sender));
												break;
											}
										}
									}
								},
							None => sender.deliver(Stage::stage_failure("ERROR: Tried to deliver a message to Stage.", &context)),
						}
					},
					Terminated(_) => (), // this should never happen
					Failure(msg, failed) =>	failed.deliver(Stage::stage_failure("ERROR: Tried to deliver failure message to Stage.", &context)),
					Undelivered(_, _) => (),
					Watch(watcher) => watcher.deliver(Stage::stage_failure("ERROR: Tried to watch the Stage.", &context)),
					Unwatch(unwatcher) => unwatcher.deliver(Stage::stage_failure("ERROR: Tried to unwatch the Stage.", &context)),
					Kill(killer) => killer.deliver(Stage::stage_failure("ERROR: Tried to kill the Stage.", &context))
				}	
			}
		});
	}
	
	fn stage_failure(err: &str, context: &Arc<Mutex<Context>>) -> CageMessage {
		Failure(box StageError::new(err), context.lock().agent()) 
	}
}

#[deriving(Clone)]
pub struct StageError {
	pub err: String
}

impl StageError {
	pub fn new(err: &str) -> StageError {
		StageError { err: err.to_string() }
	}
}
impl Message for StageError {}


