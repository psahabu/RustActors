/*
 * Contexts take care of the non-Inbox Actor machinery in the
 * Cage system. They create Actors, track the Actor's parent
 * and children, and format messages.
 */

use super::Actor;
use super::Message;
use actor_agent::Agent;
use cage_message::CageMessage;
	use cage_message::UserMessage;
	use cage_message::Terminated;
	use cage_message::Failure;
	use cage_message::Undelivered;
	use cage_message::Watch;
	use cage_message::Unwatch;
	use cage_message::Kill;

pub struct Context {
	agent: Agent,
	parent: Agent,
	children: Vec<Agent>,
}

// TODO: finish this
// static ACTOR_ROOT: Context = 

impl Context {
	/*
	 * Formatting messages for Agents.
	 * ex. agent | context.send(...)
	 *     agent | context.kill()
	 */

	// Formats a user message for an Agent.
	pub fn wrap(&self, msg: Box<Message>) -> CageMessage {
		UserMessage(msg, self.agent.clone())
	}

	// Formats a message that will tell the receiving Actor that a
	// failure occurred while consuming the message.
	pub fn failure(&self, err: Box<Message>) -> CageMessage {
		Failure(err, self.agent.clone())
	}

	// Formats a message that will tell the receiving Actor to inform
	// this Actor of its death.
	pub fn watch(&self) -> CageMessage {
		Watch(self.agent.clone())
	}

	// Formats a message that will tell the receiving Actor to
	// not inform this Actor of its death.
	pub fn unwatch(&self) -> CageMessage {
		Unwatch(self.agent.clone())
	}

	// Formats a message that will tell the receiving Actor to
	// cease receiving messages and clean up its state.
	pub fn kill(&self) -> CageMessage {
		Kill(self.agent.clone())
	}

	/*
	 * Convenient methods to access direct relatives of this Actor.
	 */
	// Returns an Agent to this Actor's parent.
	pub fn parent(&self) -> Agent {
		self.parent.clone()
	}
	// Returns a vector of Agents 
	pub fn children(&self) -> Vec<Agent> {
		self.children.clone()
	}

	/*
	 * Spins off a task for the passed Actor and places it
	 * as a child of this Actor.
	 */
	pub fn start_child<T: Actor>(&mut self) -> Agent {
		// Creation of the Actor.
		let actor: T = Actor::new();

		// Creation of the Agent.
		let (send, recv) = channel();
		let child_agent = Agent::new(send);
		self.children.push(child_agent.clone());
 		// TODO: put it in the directory

		// Creation of the Context.
		let new_ref = Context {
			agent: child_agent.clone(),
			parent: self.agent.clone(),
			children: Vec::new()
		};

		// The magnificent task that runs an Actor.
		spawn(proc() {
			actor.pre_start();
			let mut watchers = Vec::new();
			loop {
				match recv.recv() {
					// TODO: consider updating Failure to take the original message 
					UserMessage(msg, sender) => actor.receive(&new_ref, msg, sender),
					Terminated(terminated) => actor.terminated(&new_ref, terminated),
					Failure(err, sender) => actor.failure(&new_ref, err, sender),
					Undelivered(attempted, orig_msg) => actor.undelivered(&new_ref, attempted, orig_msg),
					Watch(watcher) => watchers.push(watcher),
					Unwatch(unwatcher) => {
						let mut i = 0;
						for watcher in watchers.iter() {
							if *watcher == unwatcher {
								break;
							}
							i += 1;
						}
						watchers.swap_remove(i);
					},
					Kill(killer) => {
						//TODO: override receiver drop to send Undelivered
						actor.killed(&new_ref, killer);
						for watcher in watchers.move_iter() {
							watcher.deliver(Terminated(new_ref.agent.clone()));
						}
						break;
					}
				}
			}
			actor.post_stop();		
		});

		child_agent
 	}
}
	// TODO: deal with lookups
	// NEW IDEA: hash table of Agents
	// OTHER IDEA: consider stripping out lookup 
	// 						 force users to do it manually, for safety

	/*
	pub fn lookup(&self, path: String) {

	}
	fn root_lookup(path: String) {

	}
	*/
