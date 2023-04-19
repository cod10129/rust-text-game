#![allow(dead_code)]
#![warn(unsafe_code)]

#[allow(unused_imports)]
use text_game::{
    nmsg, monologue, cutscene, sleep,
    Command as Cmd, MovementCommand as MC,
    YN::{self, Yes, No},
    Location as Loc, AreaObject as AO, Enemy,
    ObjectHolder, Format, Directional,
    process_battle, help_menu, death_msg, get_obj_user,
    Cutscene,
    Player,
    Colorize,
    Rc, RefCell, HashMap, RfcLoc
};

fn get_test_npc() -> AO {
    let text = |_: &mut Player| {
        let messages: Vec<(String, u16)> = vec![
            ("Hello!".into(), 750),
            ("Did you know that you can use shortened forms of commands?".into(), 1500),
            (format!("For example, you can type {} instead of {}.", "n".bright_yellow(), "north".bright_yellow()), 1500),
            (format!("You can even type {} to interact with things!", "i".bright_yellow()), 1500),
        ];
        monologue!("Test NPC".blue(), messages).play()
    };
    AO::new(&"Test NPC", &"An NPC for testing purposes.", text)
}

fn get_test_enemy() -> AO {
    let battle = |p: &mut Player| {
        process_battle(p, &mut Enemy::new(
            "Not Goomba".to_string(),
            5,
            |_: u16| {
                const EXTRA_DMG_CHANCE: f32 = 0.20;
                if fastrand::f32() < EXTRA_DMG_CHANCE { 
                    println!("Not Goomba did a strong attack!");
                    2 
                } else { 1 }
            },
            1,
            0.80
        ))
    };

    AO::new("Not Goomba", "An enemy for testing purposes.", battle)
}

fn boss() -> AO {
    let battle = |p: &mut Player| {
        let mut enemy = Enemy::new(
            "BOSS".red().to_string(),
            8,
            |hp: u16| {
                if hp <= 1 { 2 }
                else { 1 }
            },
            10,
            0.00
        );
        process_battle(p, &mut enemy);
        if p.health <= 0 { return; }
        sleep!(750);
        cutscene![
            ("The lock on the north door falls off.", 1000)
        ].play();
        p.flags.insert("treasure1".to_string(), true);
        p.location.borrow_mut().rem_object(&"BOSS".fmt());
    };

    AO::new("BOSS", "THE BOSS", battle)
}

fn treasure_door() -> AO {
    let open = |p: &mut Player| {
        if p.flags.contains_key("treasure1") {
            cutscene![
                ("The door slowly opens.", 1000)
            ].play();
            p.location.borrow_mut().rem_object(&"North Door".fmt());
            Loc::attach(&p.location, &Loc::new("Treasure Room", HashMap::new()), MC::North);
            let cave = p.location.w().unwrap().n().unwrap();
            Loc::attach(&p.location.n().unwrap(), &cave, MC::West);
        } else {
            cutscene![
                ("You do not have the key to this door.", 1000)
            ].play();
        }
    };

    AO::new("North Door", "A locked door that leads north.", open)
}

fn get_locations() -> RfcLoc {
    let empty_map = HashMap::new();
    let cave = Loc::new("Cave", empty_map.clone());
    let depths = Loc::new("Depths", empty_map.clone());
    let boss_room = Loc::new("Boss Room", empty_map.clone());
    boss_room.add_object(boss());
    boss_room.add_object(treasure_door());
    let village_road = Loc::new("Village Road", empty_map.clone());
    let village = Loc::new("Village", empty_map.clone());
    village.add_object(get_test_npc());
    let spawn = Loc::new("Clearing", empty_map.clone());
    spawn.add_object(
        AO::new(
            "Dev Test",
            "This object triggers test code for development purposes.",
            |_: &mut Player| {
                // INSERT TEST CODE HERE
                eprintln!("Triggering test cutscene...\n");
                get_test_cutscene().play();
            }
        )
    );
    spawn.add_object(get_test_enemy());
    Loc::attach(&spawn, &village_road, MC::West);
    Loc::attach(&village_road, &village, MC::West);
    Loc::attach(&spawn, &cave, MC::South);
    Loc::attach(&cave, &depths, MC::South);
    Loc::attach(&depths, &boss_room, MC::East);

    spawn
}

fn get_test_cutscene() -> Cutscene {
    cutscene![
        ("Welcome to the game!\n", 2000),
        ("You find yourself in a strange clearing.", 2000),
        ("There is a deep cave nearby.", 1500)
    ]
}

fn main() {
    let l = get_locations();
    let mut player = Player::new(l);

    let activate = YN::from_user("Do you want to start the game? [Y/N] ".blue().to_string().as_str());
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
            Cmd::Help => help_menu(),
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
                    println!("{}", objects.get(obj).unwrap().get_name());
                }
            },
            Cmd::Interact => {
                let objects = player.location.get_objects();
                let object = get_obj_user("interact with?", &objects);

                if object.is_none() { continue; }
                // The case of object being None is handled, so unwrap() is fine.
                object.unwrap().interact(&mut player);
                if player.health == 0 {
                    death_msg();
                    break;
                }
            },
            Cmd::Examine => {
                let objects = player.location.get_objects();
                let object = get_obj_user("examine?", &objects);

                if object.is_none() { continue; }
                let object = object.unwrap();
                println!("{}:\n", object.get_name());
                println!("{}", object.get_desc());
            }
        }
    }
}