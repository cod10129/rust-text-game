#![allow(dead_code)]
#![warn(unsafe_code)]

#[allow(unused_imports)]
use text_game::{
    fout, input,
    Command as Cmd, MovementCommand as MC,
    YN::{self, Yes, No},
    Location as Loc,
    AreaObject as AO,
    Cutscene,
    Format,
    Player,
    help_menu as display_help,
    Rc, Ref, RefCell, HashMap,
};

fn get_locations() -> Rc<RefCell<Loc>> {
    let empty_map: HashMap<String, AO> = HashMap::new();
    let cave = Loc::new("Cave", empty_map.clone());
    let depths = Loc::new("Depths", empty_map.clone());
    let boss_room = Loc::new("Boss Room", empty_map.clone());
    let treasure = Loc::new("Treasure Room", empty_map.clone());
    let spawn = Loc::new("Clearing", empty_map.clone());
    Loc::attach(&spawn, &cave, MC::South);
    Loc::attach(&cave, &depths, MC::South);
    Loc::attach(&depths, &boss_room, MC::East);
    Loc::attach(&boss_room, &treasure, MC::North);
    Loc::attach_oneway(&treasure, &cave, MC::West);

    spawn
}

fn get_test_cutscene() -> Cutscene {
    let mut scene = Cutscene::new();
    scene.add("Welcome to the game!\n", 2000);
    scene.add("You find yourself in a strange clearing.", 2000);
    scene.add("There is a deep cave nearby.", 1500);

    scene
}

#[allow(unused_variables)]
fn main() {
    let loc = get_locations();
    let mut player = Player::new(loc);

    let activate = YN::from_user("Do you want to start the game? [Y/N] ");
    if activate == No {
        println!("ok cya lol");
        return;
    }

    // Main
    loop {
        let cmd = Cmd::get();
        match cmd {
            Cmd::Quit => break,
            Cmd::North | Cmd::South | Cmd::East | Cmd::West => {
                let new = Loc::travel(
                    &player.location,
                    &cmd.clone().try_into().unwrap()
                );
                if new.is_none() {
                    println!("You cannot go {:?} of here.", cmd);
                } else {
                    player.location = Rc::new(RefCell::new(new.unwrap()));
                }
            }
            Cmd::Location => println!("You are at {:?}", player.location.borrow()),
            // TODO
            Cmd::Save => {
                println!("This feature is currently not implemented.")
            }
            Cmd::Help => display_help(),
            Cmd::Objects => {
                let objects = player.location.borrow().get_objects();
                if objects.is_empty() {
                    println!("There are no objects here.");
                    continue;
                }
                let mut keys = objects.keys().collect::<Vec<_>>();
                keys.sort_unstable();
                for obj in keys {
                    println!("{}", obj);
                }
            },
            // TODO
            Cmd::Interact => {
                println!("This feature is currently not implemented.")
            },
        }
    }
}

fn input_i32(prompt: &str) -> i32 {
    let val = input!(prompt);
    match val.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            println!("Please enter a number");
            input_i32(prompt)
        }
    }
}
