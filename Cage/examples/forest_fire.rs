#![feature(macro_rules)]
#![feature(phase)]

extern crate cage;

use cage::actor::Actor;
use cage::actor::Message;
use cage::actor_agent::Agent;
use cage::actor_context::Context;
use cage::actor_stage::Stage;

use std::rand;
use std::rand::Rng;

/*
 * Required for runtime reflection.
 */
use std::any::AnyRefExt;
#[macro_escape] mod match_any;

/*
 * Message types.
 */
#[deriving(Clone)]
struct FireSeason;
impl Message for FireSeason {}

#[deriving(Clone)]
struct Fire;
impl Message for Fire {}

/*
 * Actor types.
 */

// the Forest Actor
static LOWER_TREE_BOUND: int = 100;
static UPPER_TREE_BOUND: int = 300;
struct Forest {
  trees: int
}

impl Actor for Forest {
  // Self-contained initialization.
  fn new() -> Forest {
    Forest { trees: rand::task_rng().gen_range(LOWER_TREE_BOUND, UPPER_TREE_BOUND) }
  }

  // Uses the macro to dispatch based on msg.
  fn receive(&mut self,
             context: &mut Context,
             msg: Box<Message>,
             sender: Agent) {
    match_any! { msg match
      if FireSeason {
        _ => {
          for child in context.children().iter() {
            child.deliver(context.send(msg.clone_me()));
          }
        }
      }
      else { () }
    };
  }

  // Run before the Actor receives messages to spawn new Actors.
  fn pre_start(&mut self, context: &mut Context) {
    for _ in range(0, self.trees) {
      context.start_child::<Fir>();
    }
  }
}

// the Fir Actor.
struct Fir {
  on_fire: bool
}

impl Actor for Fir {
  fn new() -> Fir {
    Fir { on_fire: false }
  }
  fn receive(&mut self,
             context: &mut Context,
             msg: Box<Message>,
             sender: Agent) {
    match_any! { msg match
      if FireSeason {
        _ => self.on_fire = rand::task_rng().gen_range(0, 500) == 451
      },
      if Fire {
        _ => self.on_fire = true
      }
      else { () }
    };
    
    if self.on_fire {
      println!("FIRE");
      // Message broadcasting.
      context.find("../*".to_string(), box Fire);
    }
  }
}

/*
 * Top-level code.
 */
fn main() {
  // Starting the Stage and an Actor on it.
  let mut stage = Stage::new();
  let forest = stage.start::<Forest>();

  // Sending a request to an Actor for a result.
  let smokey = box FireSeason;
  let arsonist = forest.request(smokey);

  // Unpacking the response.
  arsonist.unwrap();
  println!("only you can prevent forest fires");
}
