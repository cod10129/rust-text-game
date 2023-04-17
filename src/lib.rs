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

pub use colored::Colorize;

#[macro_export]
/// Formats to {name}: {msg}
macro_rules! nmsg {
    ($name:expr, $msg:expr) => {
        format!("{}: {}", $name, $msg);
    }
}

#[macro_export]
/// monologue! takes a name (String) and messages (Vec<(String, u16)>).
/// It returns a Cutscene that contains the messages.
macro_rules! monologue {
    ($name: expr, $messages: expr ) => {{
        let mut scene = Cutscene::new();
        for (msg, wait) in $messages {
            scene.add(&nmsg!($name, msg.as_str()), wait);
        }
        scene
    }};
}

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

    pub fn add(&mut self, msg: &str, wait_ms: u16) {
        self.data.push(CutscenePart {
            msg: msg.to_string(),
            wait: Duration::from_millis(wait_ms.into()),
        })
    }

    pub fn play(&self) {
        for part in &self.data {
            println!("{}", part.msg);
            sleep(part.wait);
        }
    }
}

#[macro_export]
/// cutscene! takes some messages (String, u16), and returns
/// a cutscene containing those messages
macro_rules! cutscene {
    ( $( $message: expr ),* ) => {{
        let mut scene = Cutscene::new();
        $(
            scene.add($message.0, $message.1);
        )*
        scene
    }}
}

pub struct Player {
    pub location: Rc<RefCell<Location>>,
    pub health: u16,
    pub max_health: u16,
    pub xp: u16,
    // Different objects (like doors)
    // can access these flags to determine their state.
    // Say, defeating an enemy can set a certain flag to true,
    // which creates a connection when a door is used.
    pub flags: Vec<bool>,
}

impl Player {
    pub fn new(location: Rc<RefCell<Location>>) -> Self {
        Self {
            location,
            health: 10,
            max_health: 10,
            xp: 0,
            // 10 placeholder flags
            // For every object that uses a flag,
            // you need to check if there is space
            flags: vec![false; 10]
        }
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
        write!(f, "AreaObject {{\n\tname: {}\n\tdescription: {}\n}}", self.name, self.description)
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

    pub fn get_name(&self) -> String {
        self.name.clone()
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

    pub fn add_object(&mut self, obj: AreaObject) {
        self.objects.insert(obj.name.clone().fmt(), obj);
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
            "location" | "l" | "loc" => Some(Command::Location),
            "objects" | "o" => Some(Command::Objects),
            "interact" | "i" => Some(Command::Interact),
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
        print!("{}", "Enter a command: ".bright_green());
        fout!();

        let mut buffer = Command::Quit;
        if Command::get_buffer(&mut buffer).is_err() {
            println!("Please enter a valid command.");
            println!("Use {} to see commands.", "help".bright_yellow());
            println!();
            return Command::get();
        }
        buffer
    }
}

pub fn get_interact(objects: &HashMap<String, AreaObject>) -> Option<&AreaObject> {
    let object = input!("What do you want to interact with? ").fmt();

    match objects.get(&object) {
        Some(v) => Some(v),
        None => {
            println!("That object does not exist.");
            println!("Type {} to see all objects in this area.", "objects".bright_yellow());
            println!();
            None
        }
    }
}

pub fn help_menu() {
    let lines = [
        "--------------- HELP MENU ---------------".cyan().to_string(),
        "".to_string(),
        "Commands:".cyan().to_string(),
        "".to_string(),
        "help: displays this menu".cyan().to_string(),
        "north: moves the player north".cyan().to_string(),
        "south: moves the player south".cyan().to_string(),
        "east: moves the player east".cyan().to_string(),
        "west: moves the player west".cyan().to_string(),
        "location: displays your current location".cyan().to_string(),
        "objects: displays all objects in your current location".cyan().to_string(),
        "interact: interacts with an object in your current location".cyan().to_string(),
        format!("{} {}", "save: saves the game".cyan(), "(UNIMPLEMENTED)".red()),
        "quit: quits the game".cyan().to_string(),
    ];
    for line in lines {
        println!("{line}");
    }
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
