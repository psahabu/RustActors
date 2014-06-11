/*
 * Contexts take care of the non-Inbox Actor machinery in the
 * Cage system. They create Actors, track the Actor's parent
 * and children, and format messages.
 */
use std::rand;
use std::rand::Rng;

use actor::Actor;
use actor::Message;
use actor_agent::Agent;
use actor_agent::NAME_LENGTH;
use actor_agent::NO_ADDRESS;
use actor_agent::ROOT_ADDRESS;
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
	pub fn send(&self, msg: Box<Message:Send>) -> CageMessage {
		UserMessage(msg, self.agent.clone())
	}

	// Formats a message such that it appears to be from the
	// given Actor as opposed to this one.
	pub fn forward(&self, msg: Box<Message:Send>, from: &Agent) -> CageMessage {
		UserMessage(msg, from.clone())
	}

	// Formats a message and sends it throughout the Stage hierarchy
	// to find the designated Actor(s).
	// ex. ../* (sibling nodes)
	// 		 /blue (the node blue under root)
	pub fn find(&self, path: String, msg: Box<Message:Send>) {
		let path_tokens = path.as_slice().split('/');
		let mut sendable_path = Vec::new();

		// Push and pop tokens from the back of Vec, hence reverse.
		for token in path_tokens.rev() {
			sendable_path.push(token.to_string());
		}
 
		match sendable_path.pop() {
			Some(s) =>
				match (&s).as_slice() {
					".." =>
						self.parent.deliver(
							Find(sendable_path, msg, self.agent.clone())
						),
					_ => {
						sendable_path.push(s);
						self.root.deliver(
							Find(sendable_path, msg, self.agent.clone())
						);
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
	// Calls start_child with a random name.
	pub fn start_child<T: Actor>(&mut self) -> Agent {
		let mut rng = rand::task_rng();
		let name = rng.gen_ascii_chars().take(NAME_LENGTH).collect();
		self.start_child_name::<T>(name)
	}

	// Mends Contexts to reflect the child Actor with the given name.
	pub fn start_child_name<T: Actor>(&mut self, name: String) -> Agent {
		// Creation of the Context.
		let (send, recv) = channel::<CageMessage>();
		let context = self.child(send, name);

		// Get the child's Agent.
		let agent = context.agent();

		// Push the child's Agent onto this Actor's child list.
		self.children.push(agent.clone());

		// Consume the Receiver and Context to spawn the child.
		Context::spawn_child::<T>(recv, context);

		// Return the child's Agent.
		agent
 	}
	
	// Used to construct a child Context from a parent.
	fn child(&self, sender: Sender<CageMessage>, name: String) -> Context {
		Context {
			agent: Agent::new(sender, self.agent.path(), name), 
			parent: self.agent.clone(),
			children: Vec::new(),
			root: self.root.clone()
		}
	}

	// The magnificent function that runs an Actor.
	fn spawn_child<T: Actor>(recv: Receiver<CageMessage>, context: Context) {
		spawn(proc() {
			// Creation of the user Actor.
			let mut actor: T = Actor::new();

			// Mutable capture of Context.
			let mut context = context;

			// List of Agents watching for death.
			let mut watchers = Vec::new();

			// User Actor setup.
			actor.pre_start();

			// Receive messages and dispatch to user Actor.
			loop {
				match recv.recv() {
					UserMessage(msg, sender) => actor.receive(&mut context, msg, sender),
					Find(path, msg, sender) => {
						let mut _path = path;
						match _path.pop() {
							None => actor.receive(&mut context, msg, sender),
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
					Terminated(terminated) => actor.terminated(&mut context, terminated),
					Failure(err, failed) => actor.failed(&mut context, err, failed),
					Undelivered(attempted, orig_msg) => actor.undelivered(&mut context, attempted, orig_msg),
					Watch(watcher) => watchers.push(watcher),
					Unwatch(unwatcher) => Context::remove_unwatcher(&mut watchers, unwatcher),
					Kill(killer) => {
						// Drain and consume the receiver.
						Context::drain_recv(recv, &context);

						// Notify user Actor that it has been killed.
						actor.killed(&mut context, killer);
						
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

	// Remove the unwatcher from watchers.
	fn remove_unwatcher(watchers: &mut Vec<Agent>, unwatcher: Agent) {
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
	}

	// Drain the remaining messages from the Receiver, sending Undelivered and Terminated.
	fn drain_recv(recv: Receiver<CageMessage>, context: &Context) {
		loop {
			match recv.try_recv() {
				Ok(cage_msg) =>
					match cage_msg {
						UserMessage(orig, sender) => sender.deliver(
							Undelivered(context.agent.clone(), orig)
						),
						Find(_, orig, sender) => sender.deliver(
							Undelivered(context.agent.clone(), orig)
						),
						Watch(watcher) => watcher.deliver(
							Terminated(context.agent.clone())
						),
						_ => ()
					},
				Err(_) => ()
			}
		}
	}

	// Though publicly visible, the user can't use this due to the type of sender.
	// Used to construct a new Context for the root.
	pub fn root(sender: Sender<CageMessage>, parent: Agent) -> Context {
		let root_agent = Agent::new(sender,
																NO_ADDRESS.to_string(),
																ROOT_ADDRESS.to_string());
		Context {	
			agent: root_agent.clone(),
			parent: parent,
			children: Vec::new(),
			root: root_agent.clone()
		}
	}
}
