#![allow(dead_code)]
#![warn(unsafe_code)]

#[allow(unused_imports)]
use text_game::{
    Command as Cmd, MovementCommand as MC,
    YN::{self, Yes, No},
    Location as Loc,
    AreaObject as AO,
    get_interact,
    Cutscene,
    Format,
    Player,
    help_menu as display_help,
    Colorize,
    Rc, Ref, RefCell, HashMap,
};

fn get_locations() -> Rc<RefCell<Loc>> {
    let empty_map: HashMap<String, AO> = HashMap::new();
    let cave = Loc::new("Cave", empty_map.clone());
    let depths = Loc::new("Depths", empty_map.clone());
    let boss_room = Loc::new("Boss Room", empty_map.clone());
    let treasure = Loc::new("Treasure Room", empty_map.clone());
    let village_road = Loc::new("Village Road", empty_map.clone());
    let village = Loc::new("Village", empty_map.clone());
    let spawn = Loc::new("Clearing", empty_map.clone());
    Loc::attach(&spawn, &village_road, MC::West);
    Loc::attach(&village_road, &village, MC::West);
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

fn main() {
    let l = get_locations();
    let mut player = Player::new(l);

    let activate = YN::from_user("Do you want to start the game? [Y/N] ".blue());
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
                    let msg = format!("You cannot go {:?} of here.", cmd);
                    println!("{}", msg.red());
                    continue;
                }
                player.location = Rc::new(RefCell::new(new.unwrap()));

            }
            Cmd::Location => {
                println!("You are at {}.", player.location.borrow().to_string().bold());
            },
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
                // Sorts the objects alphabetically
                let mut keys = objects.keys().collect::<Vec<_>>();
                keys.sort_unstable();
                for obj in keys {
                    println!("{}", obj);
                }
            },
            Cmd::Interact => {
                let objects = (*player.location).clone().into_inner().get_objects();
                let object = get_interact(&objects);

                if object.is_none() { continue; }
                // The case of object being None is handled, so unwrap() is fine.
                object.unwrap().interact(&mut player);
            },
        }
    }
}