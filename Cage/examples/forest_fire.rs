#![feature(macro_rules)]
#![feature(phase)]

extern crate cage;

use cage::actor::Actor;
use cage::actor::Message;
use cage::actor_agent::Agent;
use cage::actor_context::Context;
use cage::actor_stage::Stage;

use std::any::AnyRefExt;
use std::rand;
use std::rand::Rng;

#[macro_escape] mod match_any;

#[deriving(Clone)]
struct FireSeason;

#[deriving(Clone)]
struct Fire;

impl Message for FireSeason {}
impl Message for Fire {}

struct Forest {
  trees: int
}

impl Actor for Forest {
  fn new() -> Forest {
    Forest { trees: rand::task_rng().gen_range(100, 300) }
  }
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
  fn pre_start(&mut self, context: &mut Context) {
    for _ in range(0, self.trees) {
      context.start_child::<Fir>();
    }
  }
}

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
      context.find("../*".to_string(), box Fire);
    }
  }
}

fn main() {
  let mut stage = Stage::new();
  let forest = stage.start::<Forest>();
  let smokey = box FireSeason;
  let arsonist = forest.request(smokey);
  arsonist.unwrap();
  println!("only you can prevent forest fires");
}
