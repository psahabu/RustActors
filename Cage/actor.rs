extern crate sync;

use std::any::Any;
use actor_agent::Agent;
use actor_context::Context;

pub mod actor_agent;
pub mod actor_context;
mod cage_message;

pub trait Message : Send {}

pub trait Actor {
	// TODO: reenable this, put it in the Context business area
	/*
	 * Starting an Actor from a non-Actor.
	 *
	fn start<T: Actor>() -> Agent {
		ACTOR_ROOT.start_child::<T>();
	}
	*/

	// Requires that the Actor be constructed in such a way that
	// it owns all of its memory.
	fn new() -> Self; 

	/*
	 * The main function.
	 */
	fn receive(&self,
						 context: &Context,
						 msg: Box<Message>,
						 sender: Agent);
	
	/*
	 * Handling errors from other Actors.
	 */

	// Called when another Actor dies, if this Actor was watching for the
	// other's death.
	fn terminated(&self,
								context: &Context,
								terminated: Agent) {}
	
	// Called if a message sent by this Actor causes failure in another.
	fn failure(&self,
						 context: &Context,
						 err: Box<Message>,
						 sender: Agent) {}

	// Called if a message sent by this Actor cannot be delivered.
	fn undelivered(&self,
								 context: &Context,
								 target: Agent,
								 orig_msg: Box<Message>) {}

	/*
	 * Setup, last licks, teardown.
	 */

	// Called before this Actor starts receiving messages.
	fn pre_start(&self) {}

	// Actors are responsible for recovery from their own errors.
	
	// Called just after an actor is killed, with the ability to
	// reply to the killer.
	fn killed(&self,
						context: &Context,
						killer: Agent) {}
	
	// Called after this Actor permanently ceases receiving messages.
	fn post_stop(&self) {}
}
