#![allow(dead_code)]
#![warn(unsafe_code)]

#[allow(unused_imports)]
use text_game::{fout, input,
           YN::{self, Yes, No},
           Command as Cmd,
           MovementCommand as MC,
           Location as Loc,
           Cutscene,
           help_menu as display_help,
           Rc, RefCell, Ref,
           Format};

fn get_locations() -> Rc<RefCell<Loc>> {
  let cave = Loc::new("Cave");
  let depths = Loc::new("Depths");
  let boss_room = Loc::new("BOSS Room");
  let treasure = Loc::new("Treasure Room");
  let spawn = Loc::new("Clearing");
  Loc::attach(&spawn, &cave, MC::South);
  Loc::attach(&cave, &depths, MC::South);
  Loc::attach(&depths, &boss_room, MC::East);
  Loc::attach(&boss_room, &treasure, MC::North);
  Loc::attach_oneway(&treasure, &cave, MC::West);
  
  spawn
}

fn get_start_cutscene() -> Cutscene {
  let mut scene = Cutscene::new();
  scene.add("Welcome to the game!\n", 2000);
  scene.add("You find yourself in a strange clearing.", 2000);
  scene.add("There is a deep cave nearby.", 1500);
  
  scene
}

#[allow(unused_variables)]
fn main() {
  let mut locmap = get_locations();
  
  let activate = YN::from_user("Do you want to start the game? [Y/N] ");
  if activate == No {
    println!("ok cya lol");
    return
  }
  
  // Main
  loop {
    let cmd = Cmd::get();
    match cmd {
      Cmd::Quit => break,
      Cmd::North | Cmd::South | Cmd::East | Cmd::West => {
        let loc = (*locmap).clone().into_inner();
        let new = Loc::travel(&locmap, &cmd.clone().try_into().unwrap());
        if new.is_none() {
          println!("You cannot go {:?} of here.", cmd);
        } else {
          locmap = Rc::new(RefCell::new(new.unwrap()));
        }
      },
      Cmd::Location => println!("You are at {:?}", (*locmap).clone().into_inner()),
      Cmd::Save => { 
        get_start_cutscene().play();
        // println!("This feature is currently not implemented.") 
      },
      Cmd::Help => display_help(),
    }
  }
}

fn input_i32(prompt: &str) -> i32 {
  let val = input!(prompt);
  match val.trim().parse() {
    Ok(v) => return v,
    Err(_) => {
      println!("Please enter a number");
      return input_i32(prompt);
    }
  }
}
