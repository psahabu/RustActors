/*
 * Defines the different kinds of messages that Agents and
 * ActorRefs handle for their Actors.
 */
use super::Message;
use actor_agent::Agent;

pub enum CageMessage {
	UserMessage(Box<Message>, Agent),
	Terminated(Agent),
	Failure(Box<Message>, Agent),
	Undelivered(Agent, Box<Message>),
	Watch(Agent),
	Unwatch(Agent),
	Kill(Agent)
}
