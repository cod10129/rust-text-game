#![allow(dead_code)]
mod input;

use std::fmt;
pub use std::cell::Ref;
pub use std::cell::RefCell;
pub use std::ops::Deref;
pub use std::rc::Rc;
pub use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

struct CutscenePart {
    msg: String,
    wait: Duration,
}

pub struct Cutscene {
    data: Vec<CutscenePart>,
}

impl Cutscene {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, msg: &str, wait_millis: u16) {
        self.data.push(CutscenePart {
            msg: msg.to_string(),
            wait: Duration::from_millis(wait_millis.into()),
        })
    }

    pub fn play(&self) {
        for part in &self.data {
            println!("{}", part.msg);
            sleep(part.wait);
        }
    }
}

pub struct Player {
    pub location: Rc<RefCell<Location>>,
}

impl Player {
    pub fn new(location: Rc<RefCell<Location>>) -> Self {
        Self { location }
    }
}

#[derive(Clone)]
pub struct AreaObject {
    name: String,
    description: String,
    func: fn(&mut Player),
}

impl fmt::Debug for AreaObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for AreaObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n\n{}", self.name, self.description)
    }
}

impl AreaObject {
    pub fn new(name: &str, description: &str, func: fn(&mut Player)) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            func
        }
    }

    pub fn interact(&self, player: &mut Player) {
        (self.func)(player);
    }
}

#[derive(Clone)]
pub struct Location {
    n: Option<Rc<RefCell<Location>>>,
    s: Option<Rc<RefCell<Location>>>,
    w: Option<Rc<RefCell<Location>>>,
    e: Option<Rc<RefCell<Location>>>,
    name: String,
    objects: HashMap<String, AreaObject>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Location {
    pub fn new(name: &str, objects: HashMap<String, AreaObject>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            n: None,
            s: None,
            w: None,
            e: None,
            name: name.to_string(),
            objects,
        }))
    }

    pub fn set_n(&mut self, other: &Rc<RefCell<Location>>) {
        self.n = Some(Rc::clone(other));
    }

    pub fn set_s(&mut self, other: &Rc<RefCell<Location>>) {
        self.s = Some(Rc::clone(other));
    }

    pub fn set_w(&mut self, other: &Rc<RefCell<Location>>) {
        self.w = Some(Rc::clone(other));
    }

    pub fn set_e(&mut self, other: &Rc<RefCell<Location>>) {
        self.e = Some(Rc::clone(other));
    }

    pub fn traverse(&self, cmd: &MovementCommand) -> Option<Self> {
        use MovementCommand::*;
        let new = match cmd {
            North => &self.n,
            South => &self.s,
            East => &self.e,
            West => &self.w,
        };
        if new.is_some() {
            // What happens here:
            // new is &Option<Rc<RefCell<Location>>>
            // Option.as_ref() returns Option<&Rc<RefCell<Location>>>
            // the value INSIDE the option (&Rc<RefCell<Location>>)
            // is cloned and extracted from to return an Option<Location>
            new.as_ref().map(|x| (**x).clone().into_inner())
        } else {
            None
        }
    }

    pub fn travel(locmap: &Rc<RefCell<Location>>, direction: &MovementCommand) -> Option<Location> {
        locmap.borrow().traverse(direction)
    }

    pub fn add_object(&mut self, name: String, description: String, func: fn(&mut Player)) {
        let new = AreaObject::new(&name, &description, func);
        self.objects.insert(name, new);
    }

    pub fn get_object(&self, name: &str) -> Option<AreaObject> {
        self.objects.get(name).cloned()
    }

    pub fn get_objects(&self) -> HashMap<String, AreaObject> {
        self.objects.clone()
    }

    /// Attaches loc to other.
    /// This function sets loc.dir to other, and other.dir.flip() to loc
    pub fn attach(
        loc: &Rc<RefCell<Location>>,
        other: &Rc<RefCell<Location>>,
        dir: MovementCommand,
    ) {
        Location::attach_oneway(loc, other, dir.clone());
        Location::attach_oneway(other, loc, dir.flip());
    }

    pub fn attach_oneway(
        loc: &Rc<RefCell<Location>>,
        other: &Rc<RefCell<Location>>,
        dir: MovementCommand,
    ) {
        let mut l = loc.borrow_mut();
        match dir {
            MovementCommand::North => l.set_n(other),
            MovementCommand::South => l.set_s(other),
            MovementCommand::East => l.set_e(other),
            MovementCommand::West => l.set_w(other),
        }
    }
}

pub use YN::*;
#[derive(Debug, PartialEq)]
pub enum YN {
    Yes,
    No,
}

impl YN {
    pub fn from_string(string: String) -> Option<YN> {
        match string.as_str() {
            "y" | "yes" => Some(Yes),
            "n" | "no" => Some(No),
            _ => None,
        }
    }

    pub fn from_user(prompt: &str) -> YN {
        let string = input!(prompt).fmt();
        match YN::from_string(string) {
            Some(v) => v,
            None => {
                println!("Please enter Y or N.");
                Self::from_user(prompt)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    North,
    South,
    East,
    West,
    Help,
    Location,
    Objects,
    Interact,
    Save,
    Quit,
}

impl Command {
    pub fn from_str(string: String) -> Option<Command> {
        match string.as_str() {
            "north" | "n" => Some(Command::North),
            "south" | "s" => Some(Command::South),
            "east" | "e" => Some(Command::East),
            "west" | "w" => Some(Command::West),
            "help" => Some(Command::Help),
            "location" | "l" => Some(Command::Location),
            "objects" => Some(Command::Objects),
            "interact" => Some(Command::Interact),
            "save" => Some(Command::Save),
            "quit" | "exit" | "close" => Some(Command::Quit),
            _ => None,
        }
    }

    pub fn get_buffer(buffer: &mut Command) -> Result<(), ()> {
        let string = input!().fmt();
        match Command::from_str(string) {
            Some(v) => {
                *buffer = v;
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn get() -> Command {
        print!("Enter a command: ");
        fout!();

        let mut buffer = Command::Quit;
        if Command::get_buffer(&mut buffer).is_err() {
            println!("Please enter a valid command.");
            println!("Use help to see commands.");
            println!();
            return Command::get();
        }
        buffer
    }
}

pub fn get_interact(objects: &HashMap<String, AreaObject>) -> Option<&AreaObject> {
    let object = input!("What do you want to interact with? ").fmt();
    // Give an escape option for the user to cancel the interaction.
    if object == String::from("cancel") { return None; }

    match objects.get(&object) {
        Some(v) => Some(v),
        None => {
            println!("That object does not exist.");
            println!("Type objects to see all objects in this area.");
            get_interact(objects)
        }
    }
}

pub fn help_menu() {
    let msg = "
--------------- HELP MENU ---------------

Commands:

help: displays this menu
north: moves the player north
south: moves the player south
east: moves the player east
west: moves the player west
location: displays your current location
save: saves the game (UNIMPLEMENTED)
quit: quits the game
  ";
    println!("{msg}");
}

#[derive(Debug, Clone)]
pub enum MovementCommand {
    North,
    South,
    East,
    West,
}

impl TryFrom<Command> for MovementCommand {
    type Error = ();
    fn try_from(value: Command) -> Result<Self, Self::Error> {
        use Command as Cmd;
        match value {
            Cmd::North => Ok(Self::North),
            Cmd::South => Ok(Self::South),
            Cmd::East => Ok(Self::East),
            Cmd::West => Ok(Self::West),
            _ => Err(()),
        }
    }
}

impl MovementCommand {
    pub fn flip(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    pub fn flip_in_place(&mut self) {
        *self = self.flip();
    }
}

pub trait Format {
    fn fmt(&self) -> String;
}
impl Format for String {
    fn fmt(&self) -> String {
        self.trim().to_ascii_lowercase()
    }
}
