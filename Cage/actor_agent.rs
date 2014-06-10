/*
 * Agents are the implementation of an Inbox
 * for an Actor in the Cage system.
 */
use std::comm::channel;
use std::comm::Sender;
use sync::Future;

use super::Message;
use cage_message::CageMessage;
	use cage_message::UserMessage;
	use cage_message::Find;
	use cage_message::Terminated;
	use cage_message::Failure;
	use cage_message::Undelivered;
	use cage_message::Watch;

#[deriving(Clone)]
pub struct Agent {
	inbox: Sender<CageMessage>,
	path: String,
	name: String
}

impl Agent {
		// Instructs the Agent to deliver the message to the Actor.
	pub fn deliver(&self, msg: CageMessage) {
		match self.inbox.send_opt(msg) {
			Err(err) => match err {
				UserMessage(orig, sender) => sender.deliver(Undelivered(self.clone(), orig)),
				Find(_, orig, sender) => sender.deliver(Undelivered(self.clone(), orig)),
				Watch(watcher) => watcher.deliver(Terminated(self.clone())),
				_ => ()
			},
			_ => ()
		}
	}

	// For message sending from a non-Actor.
	pub fn request(&self, msg: Box<Message:Send>)
			-> Future<Result<Box<Message:Send>, Option<Box<Message:Send>>>> {
		let (send, recv) = channel();
		self.deliver(UserMessage(msg, Agent::new(send)));
		Future::from_fn(proc() {
			match recv.recv() {
				UserMessage(msg, _) => Ok(msg),
				Failure(err, _) => Err(Some(err)),
				_ => Err(None)
			}
		})
	}

	// Returns the path of this Actor.
	pub fn path(&self) -> String {
		self.path.clone()
	}

	// Returns the name of this Actor (path-independent).
	pub fn name(&self) -> String {
		self.name.clone()
	}
	
	pub fn new(sender: Sender<CageMessage>) -> Agent {
		Agent { inbox: sender, path: "egalite".to_string(), name: "inigo montoya".to_string()}
	}
	// TODO: actually implement this, getting name from Context  
	// TODO: figure out a way to give out guaranteed unique names
	// 			 	 -idea: countup atomic integer
	// 			 	 -idea: check Google
	/*
	pub fn new(sender: Sender<CageMessage>, dir: String, name: String) -> Agent {
	}
	*/

}

impl Eq for Agent { }
impl PartialEq for Agent {
	fn eq(&self, other: &Agent) -> bool {
		self.path == other.path
	}
}

/*
impl BitOr<CageMessage, ()> for Agent {
	fn bitor(&self, msg: &CageMessage) {
		match self.inbox.send_opt(*msg) {
			Err(err) => match err {
				UserMessage(orig, sender) => sender | Undelivered(self.clone(), orig),
				Watch(watcher) => watcher | Terminated(self.clone()),
				_ => ()
			},
			_ => ()
		}
	}
}
*/
