/*
 * Defines the different kinds of messages that Agents and
 * ActorRefs handle for their Actors.
 */
use actor_agent::Agent;

pub enum CageMessage {
	Message(Box<Send>, Agent),
	Terminated(Agent),
	Failure(Box<Send>, Agent),
	Undelivered(Agent, Box<Send>),
	Watch(Agent),
	Unwatch(Agent),
	Kill(Agent)
}
