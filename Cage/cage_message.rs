/*
 * Defines the different kinds of messages that Agents and
 * ActorRefs handle for their Actors.
 */
use actor::Message;
use actor_agent::Agent;

pub enum CageMessage {
	UserMessage(Box<Message:Send>, Agent),
	Find(Vec<String>, Box<Message:Send>, Agent),
	Terminated(Agent),
	Failure(Box<Message:Send>, Agent),
	Undelivered(Agent, Box<Message:Send>),
	Watch(Agent),
	Unwatch(Agent),
	Kill(Agent)
}
