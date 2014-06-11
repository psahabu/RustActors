/*
 * Agents are the implementation of an Inbox
 * for an Actor in the Cage system.
 */
use std::any::Any;
use std::comm::channel;
use std::comm::Sender;
use sync::Future;

use actor::Message;
use cage_message::CageMessage;
  use cage_message::UserMessage;
  use cage_message::Find;
  use cage_message::Terminated;
  use cage_message::Failure;
  use cage_message::Undelivered;
  use cage_message::Watch;

pub static NO_ADDRESS: &'static str = "";
pub static ROOT_ADDRESS: &'static str = "/";
pub static NAME_LENGTH: uint = 20;

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
        UserMessage(orig, sender) => sender.deliver(
          Undelivered(self.clone(), orig)
        ),
        Find(_, orig, sender) => sender.deliver(
          Undelivered(self.clone(), orig)
        ),
        Watch(watcher) => watcher.deliver(
          Terminated(self.clone())
        ),
        _ => ()
      },
      _ => ()
    }
  }

  // For message sending from a non-Actor.
  pub fn request(&self, msg: Box<Message:Send>) -> Future<Option<Box<Message:Send>>> {
    let (send, recv) = channel();
    self.deliver(UserMessage(msg.clone_me(), Agent::new(send,
                                                        NO_ADDRESS.to_string(),
                                                        NO_ADDRESS.to_string())));
    Future::from_fn(proc() {
      match recv.recv_opt() {
        Ok(m) =>
          match m {
            UserMessage(msg, _) => Some(msg),
            Failure(err, _) => Some(msg),
            _ => None
          },
        _ => None
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
  
  // Returns a new Agent with a given name.
  pub fn new(sender: Sender<CageMessage>, dir: String, name: String) -> Agent {
    Agent {
      inbox: sender,
      path: dir.append(name.as_slice()),
      name: name
    }
  }
}

impl Eq for Agent { }
impl PartialEq for Agent {
  fn eq(&self, other: &Agent) -> bool {
    self.path == other.path
  }
}
