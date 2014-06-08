extern crate sync;

use std::any::Any;
use actor_agent::Agent;
use actor_context::Context;

pub mod actor_agent;
pub mod actor_context;
mod cage_message;
// static mut ACTOR_ROOT: Context  = // TODO: construct this in context.rs

pub trait Actor {
	// TODO: reenable this
	/*
	 * Starting an Actor from a non-Actor.
	 *
	fn start(actor: Box<Actor>) -> Agent {
		ACTOR_ROOT.start_child(actor)
	}
	*/

	/*
	 * The main function.
	 */
	fn receive(&self,
						 context: &Context,
						 msg: Box<Any>,
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
						 err: Box<Any>,
						 sender: Agent) {}

	// Called if a message sent by this Actor cannot be delivered.
	fn undelivered(&self,
								 context: &Context,
								 target: Agent,
								 orig_msg: Box<Any>) {}

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
