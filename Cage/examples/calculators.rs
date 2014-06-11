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
struct Rounds {
  rounds: int
}

#[deriving(Clone)]
struct WantNumber;

#[deriving(Clone)]
struct AddNum {
  num: int
}

#[deriving(Clone)]
struct Sum {
  sum: int
}

impl Message for Rounds {}
impl Message for WantNumber {}
impl Message for AddNum {}
impl Message for Sum {}

struct Generator {
  boss: Option<Agent>,
  first: Option<int>
}

impl Actor for Generator {
  fn new() -> Generator {
    Generator {
      boss: None,
      first: None
    }
  }
  fn receive(&mut self,
             context: &mut Context,
             msg: Box<Message>,
             sender: Agent) {
    match_any! { msg match
      if Rounds {
        &Rounds{ rounds } => {
           self.boss = Some(sender.clone());
           for child in context.children().iter() {
             child.deliver(context.send(msg.clone_me()));
           }
        }
      },
      if WantNumber {
        _ => sender.deliver(context.send(box AddNum { num: rand::task_rng().gen_range(0, 10) }))
      },
      if Sum {
        &Sum{ sum: i } =>
          match self.first {
            Some(j) =>
              match self.boss {
                Some(ref b) => b.deliver(context.send(box Sum { sum: i+j })),
                None => () // this should never happen
              },
            None => self.first = Some(i)
          }
      }
      else {
        ()
      }
    };
  }
  fn pre_start(&mut self, context: &mut Context) {
    context.start_child::<Calculator>();
    context.start_child::<Calculator>();
  }
}

struct Calculator {
  sum: int,
  name: String,
  rounds: int
}

impl Actor for Calculator {
  fn new() -> Calculator {
    Calculator {
      sum: 0,
      name: "Nicolas Cage".to_string(),
      rounds: 0
    }
  }
  fn receive(&mut self,
             context: &mut Context,
             msg: Box<Message>,
             sender: Agent) {
    match_any! { msg match
      if Rounds {
        &Rounds{ rounds } => {
          self.rounds = rounds
        }
      },
      if AddNum {
        &AddNum{ num } => {
          self.sum += num;
          self.rounds -= 1;
        }
      }
      else {
        ()
      }
    };

    if self.rounds > 0 {
      sender.deliver(context.send(box WantNumber)); 
    } else {
      sender.deliver(context.send(box Sum { sum: self.sum }));
    }
  }
}

fn main() {
  let mut stage = Stage::new();
  let gen = stage.start::<Generator>();
  let msg = box Rounds { rounds: 10 };
  let winner = gen.request(msg);
  match winner.unwrap() {
    Some(msg) => {
      match_any! { msg match
        if Sum {
          &Sum{ sum } => println!("The calculators summed to {}", sum)
        }
        else {
          println!("Both calculators malfunctioned.")
        }
      };
    } ,
    None => println!("Generator malfunctioned.")
  }
}
