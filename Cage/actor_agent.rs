/*
 * Agents are the implementation of an Inbox
 * for an Actor in the Cage system.
 */
use std::any::Any;
use std::comm::channel;
use std::comm::Sender;
use sync::Future;
use cage_message::CageMessage;
	use cage_message::Message;
	use cage_message::Terminated;
	use cage_message::Failure;
	use cage_message::Undelivered;
	use cage_message::Watch;

#[deriving(Clone)]
pub struct Agent {
	inbox: Sender<CageMessage>,
	path: String
}

impl Agent {
	pub fn new(sender: Sender<CageMessage>) -> Agent {
		Agent { inbox: sender, path: "egalite".to_string() }
	}
	// TODO: actually implement this, getting name from ActorRef  
	// TODO: figure out a way to give out guaranteed unique names
	// 			 	 -idea: countup atomic integer
	// 			 	 -idea: check Google
	/*
	pub fn new(sender: Sender<CageMessage>, dir: String, name: String) -> Agent {
	}
	*/
	pub fn get_path(&self) -> String {
		self.path.clone()
	}
	pub fn request(&self, msg: Box<Send>)
			-> Future<Result<Box<Any>, Option<Box<Any>>>> {
		let (send, recv) = channel();
		self | Message(msg, Agent::new(send));
		Future::from_fn(proc() {
			match recv.recv() {
				Message(msg, _) => Ok(msg),
				Failure(err, _) => Err(Some(err)),
				_ => Err(None)
			}
		})
	}
}

impl BitOr<CageMessage, ()> for Agent {
	pub fn bitor(&self, msg: CageMessage) {
		match self.inbox.send_opt(msg) {
			Err(err) => match err {
				Message(orig, sender) => sender | Undelivered(self.clone(), orig),
				Watch(watcher) => watcher | Terminated(self.clone()),
				_ => ()
			}
			// TODO: cleanup sender from agent directory
			// 			 need to repeat this Message/Watch match on bad lookups
			// agent directory could itself be Actor-like
		}
	}
}

impl Eq for Agent { }
impl PartialEq for Agent {
	fn eq(&self, other: &Agent) -> bool {
		self.path == other.path
	}
}

