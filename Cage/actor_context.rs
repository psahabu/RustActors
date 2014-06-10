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

impl Context {
	/*
	 * Formatting messages for Agents.
	 * ex. agent.deliver(context.send(...))
	 *     agent.deliver(context.kill())
	 */

	// Formats a user message for an Agent.
	pub fn wrap(&self, msg: Box<Message:Send>) -> CageMessage {
		UserMessage(msg, self.agent.clone())
	}

	// Formats a message such that it appears to be from the
	// given Actor as opposed to this one.
	pub fn forward(&self, msg: Box<Message:Send>, from: &Agent) -> CageMessage {
		UserMessage(msg, from.clone())
	}

	// Formats a message that will tell the receiving Actor that a
	// failure occurred while consuming the message.
	pub fn failure(&self, err: Box<Message:Send>) -> CageMessage {
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
	 * Convenience methods to access direct relatives of this Actor.
	 */
	// Returns the Agent for this Actor.
	pub fn agent(&self) -> Agent {
		self.agent.clone()
	}
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
		// Creation of the Context.
		let (send, recv) = channel::<CageMessage>();
		let context = Context::new(send, self.agent.clone());

		// Get the child's Agent.
		let agent = context.agent();

		// Push the child's Agent onto this Actor's child list.
		self.children.push(agent.clone());
	
		// The magnificent task that runs an Actor.
		spawn(proc() {
			// Creation of the user Actor.
			let actor: T = Actor::new();

			// List of Agents watching for death.
			let mut watchers = Vec::new();

			// User Actor setup.
			actor.pre_start();

			// Receive messages and dispatch to user Actor.
			loop {
				match recv.recv() {
					// TODO: consider updating Failure to take the original message 
					UserMessage(msg, sender) => actor.receive(&context, msg, sender),
					Terminated(terminated) => actor.terminated(&context, terminated),
					Failure(err, failed) => actor.failed(&context, err, failed),
					Undelivered(attempted, orig_msg) => actor.undelivered(&context, attempted, orig_msg),
					Watch(watcher) => watchers.push(watcher),
					Unwatch(unwatcher) => {
						let mut i = 0;
						let mut remove = false;
						for watcher in watchers.iter() {
							if *watcher == unwatcher {
								remove = true;
								break;
							}
							i += 1;
						}
						if remove {
							watchers.swap_remove(i);
						}
					},
					Kill(killer) => {
						// TODO: try to get the remaining messages, send Undelivered
						// use recv_opt until it fails

						// Stop receiving messsages immediately.
						drop(recv);

						// Notify user Actor that it has been killed.
						actor.killed(&context, killer);
						
						// Continue cleanup.
						break;
					}
				}
			}
			
			// Reap the children.
			for child in context.children().move_iter() {
				child.deliver(Kill(context.agent()));
			}	

			// Notify watchers of death.
			for watcher in watchers.move_iter() {
				watcher.deliver(Terminated(context.agent.clone()));
			}

			// User Actor cleanup.
			actor.post_stop();		
		});

		// Return the child's Agent.
		agent
 	}

	pub fn new(sender: Sender<CageMessage>, parent: Agent) -> Context {
		Context {	
			agent: Agent::new(sender),
			parent: parent,
			children: Vec::new()
		}
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
