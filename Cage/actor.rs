use std::any::Any;
use std::any::AnyRefExt;
use std::intrinsics::TypeId;
use std::mem::transmute;
use std::mem::transmute_copy;
use std::raw::TraitObject;

use actor_agent::Agent;
use actor_context::Context;

pub trait Message : Send + Clone + Any {
  // A work around suggested by Huon to enable cloning of trait objects
  fn clone_me(&self) -> Box<Message:Send> {
    box self.clone() as Box<Message:Send>
  }
}

// copied from std::any, like Chris Morgan's HTTP headers in Teepee
impl<'a> AnyRefExt<'a> for &'a Message {
  #[inline]
  fn is<T: 'static>(self) -> bool {
    let t = TypeId::of::<T>();
    let boxed = self.get_type_id();
    t == boxed
  }

  #[inline]
  fn as_ref<T: 'static>(self) -> Option<&'a T> {
    if self.is::<T>() {
      unsafe {
        let to: TraitObject = transmute_copy(&self);
        Some(transmute(to.data))
      }
    } else {
      None
    }
  }
}


pub trait Actor {
  // Requires that the Actor be constructed in such a way that
  // it owns all of its memory.
  fn new() -> Self; 

  /*
   * The main function.
   */
  fn receive(&mut self,
             context: &mut Context,
             msg: Box<Message>,
             sender: Agent);
  
  /*
   * Handling errors from other Actors.
   */

  // Called when another Actor dies, if this Actor was watching for the
  // other's death.
  fn terminated(&mut self,
                context: &mut Context,
                terminated: Agent) {}
  
  // Called if a message sent by this Actor causes failure in another.
  fn failed(&mut self,
            context: &mut Context,
            err: Box<Message>,
            failed: Agent) {}

  // Called if a message sent by this Actor cannot be delivered.
  fn undelivered(&mut self,
                 context: &mut Context,
                 target: Agent,
                 orig_msg: Box<Message>) {}

  /*
   * Setup, last licks, teardown.
   */

  // Called before this Actor starts receiving messages.
  fn pre_start(&mut self) {}

  // Actors are responsible for recovery from their own errors.
  
  // Called just after an actor is killed, with the ability to
  // reply to the killer.
  fn killed(&mut self,
            context: &mut Context,
            killer: Agent) {}
  
  // Called after this Actor permanently ceases receiving messages.
  fn post_stop(&mut self) {}
}
