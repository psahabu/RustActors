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
	use cage_message::Find;
	use cage_message::Terminated;
	use cage_message::Failure;
	use cage_message::Undelivered;
	use cage_message::Watch;
	use cage_message::Unwatch;
	use cage_message::Kill;

#[deriving(Clone)]
pub struct Context {
	agent: Agent,
	parent: Agent,
	children: Vec<Agent>,
	root: Agent
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

	pub fn find(&self, path: String, msg: Box<Message:Send>) {
		let path_tokens = path.as_slice().split('/');
		let mut sendable_path = Vec::new();

		// Push and pop tokens from the back, hence reverse.
		for token in path_tokens.rev() {
			sendable_path.push(token.to_string());
		}
 
		match sendable_path.pop() {
			Some(s) =>
				match &s.as_slice() {
					".." => self.parent.deliver(Find(sendable_path, msg, self.agent.clone())),
					_ => {
						sendable_path.push(s);
						self.root.deliver(Find(sendable_path, msg, self.agent.clone()));
					}
				},
			None => ()
		}
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
	// Mends Contexts to reflect the child Actor.
	pub fn start_child<T: Actor>(&mut self) -> Agent {
		// Creation of the Context.
		let (send, recv) = channel::<CageMessage>();
		let context = self.child(send);

		// Get the child's Agent.
		let agent = context.agent();

		// Push the child's Agent onto this Actor's child list.
		self.children.push(agent.clone());

		// Consume the Receiver and Context to spawn the child.
		Context::spawn_child::<T>(recv, context);

		// Return the child's Agent.
		agent
 	}
	
	// The magnificent function that runs an Actor.
	fn spawn_child<T: Actor>(recv: Receiver<CageMessage>, context: Context) {
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
					UserMessage(msg, sender) => actor.receive(&context, msg, sender),
					Find(path, msg, sender) => {
						let mut _path = path;
						match _path.pop() {
							None => actor.receive(&context, msg, sender),
							Some(ref s) =>
								match s.as_slice() {
									"*" => { 
										for child in context.children.iter() {
											child.deliver(UserMessage(msg.clone_me(), sender.clone()));
										}	
									},
									".." => context.parent.deliver(Find(_path, msg, sender)),
									_ => {
										for child in context.children.iter() {
											if child.name() == *s {
												child.deliver(Find(_path, msg, sender));
												break;
											}
										}
									}
								}
						}
					},
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
						// Drain the remaining messages, sending Undelivered and Terminated.
						loop {
							match recv.try_recv() {
								Ok(cage_msg) =>
									match cage_msg {
										UserMessage(orig, sender) => sender.deliver(Undelivered(context.agent.clone(), orig)),
										Find(_, orig, sender) => sender.deliver(Undelivered(context.agent.clone(), orig)),
										Watch(watcher) => watcher.deliver(Terminated(context.agent.clone())),
										_ => ()
									},
								Err(_) => ()
							}
						}

						// Stop receiving messsages immediately.
						drop(recv);

						// Notify user Actor that it has been killed.
						actor.killed(&context, killer);
						
						// Continue cleanup.
						break;
					}
				}
			}
		
			// Notify watchers of this Actor's death.
			for watcher in watchers.move_iter() {
				watcher.deliver(Terminated(context.agent.clone()));
			}
	
			// Reap this Actor's children.
			for child in context.children().iter() {
				child.deliver(Kill(context.agent()));
			}	

			// User Actor cleanup.
			actor.post_stop();		
		});
	}

	// Though publicly visible, the user can't use this due to the type of sender.
	// Used to construct a new Context for the root.
	pub fn root(sender: Sender<CageMessage>, parent: Agent) -> Context {
		let root_agent = Agent::new(sender);
		Context {	
			agent: root_agent.clone(),
			parent: parent,
			children: Vec::new(),
			root: root_agent.clone() 
		}
	}

	// Used to construct a child Context from a parent.
	fn child(&self, sender: Sender<CageMessage>) -> Context {
		let child_agent = Agent::new(sender);
		Context {
			agent: child_agent,
			parent: self.agent.clone(),
			children: Vec::new(),
			root: self.root.clone()
		}
	}
}
