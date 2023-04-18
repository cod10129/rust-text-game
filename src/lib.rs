#![allow(dead_code)]
mod input;

use std::fmt;
pub use std::cell::RefCell;
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

/// cmd_desc! takes a name and a description that are format!-able.
/// The command becomes bright_yellow, and the description cyan.
macro_rules! cmd_desc {
    ($name: expr, $desc: expr) => {
        format!("{}{} {}", $name.bright_yellow(), ":".bright_yellow(), $desc.cyan())
    }
}

/// pall! prints all the values in an Iterator.
macro_rules! pall {
    ($iter: expr) => {
        for item in $iter {
            println!("{}", item);
        }
    }
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
/// a cutscene containing those messages.
macro_rules! cutscene {
    ( $( $message: expr ),* ) => {{
        let mut scene = Cutscene::new();
        $(
            scene.add($message.0, $message.1);
        )*
        scene
    }}
}

pub enum Weapon {
    Wooden,
    Metal
}

impl Weapon {
    pub fn damage(&self) -> u16 {
        match self {
            Weapon::Wooden => 1,
            Weapon::Metal => 2
        }
    }
}

pub struct Player {
    pub location: Rc<RefCell<Location>>,
    pub health: u16,
    pub max_health: u16,
    pub xp: u16,
    pub weapon: Weapon,
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
            weapon: Weapon::Wooden,
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

pub struct Enemy {
    name: String,
    health: u16,
    damage: fn() -> u16,
    xp: u16,
    can_run: bool,
}

impl Enemy {
    pub fn new(
        name: String,
        health: u16,
        damage: fn() -> u16,
        xp: u16,
        can_run: bool
    ) -> Self {
        Enemy { name, health, damage, xp, can_run }
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

pub trait ObjectHolder {
    fn add_object(&self, obj: AreaObject);
    fn get_object(&self, name: &str) -> Option<AreaObject>;
    fn get_objects(&self) -> HashMap<String, AreaObject>;
}

impl ObjectHolder for Rc<RefCell<Location>> {
    fn add_object(&self, obj: AreaObject) {
        self.borrow_mut().add_object(obj);
    }

    fn get_object(&self, name: &str) -> Option<AreaObject> {
        self.borrow().get_object(name)
    }

    fn get_objects(&self) -> HashMap<String, AreaObject> {
        self.borrow().get_objects()
    }
}

pub use YN::*;
#[derive(Debug, PartialEq)]
pub enum YN {
    Yes,
    No,
}

impl TryFrom<String> for YN {
    type Error = ();
    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.as_str() {
            "y" | "yes" => Ok(Yes),
            "n" | "no" => Ok(No),
            _ => Err(()),
        }
    }
}

impl YN {
    pub fn from_user(prompt: &str) -> YN {
        let string = input!(prompt).fmt();
        match string.try_into() {
            Ok(v) => v,
            Err(_) => {
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

impl TryFrom<String> for Command {
    type Error = ();
    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.as_str() {
            "north" | "n" => Ok(Command::North),
            "south" | "s" => Ok(Command::South),
            "east" | "e" => Ok(Command::East),
            "west" | "w" => Ok(Command::West),
            "help" => Ok(Command::Help),
            "location" | "l" | "loc" => Ok(Command::Location),
            "objects" | "o" => Ok(Command::Objects),
            "interact" | "i" => Ok(Command::Interact),
            "save" => Ok(Command::Save),
            "quit" | "exit" | "close" => Ok(Command::Quit),
            _ => Err(()),
        }
    }
}

impl Command {
    pub fn get_buffer(buffer: &mut Command) -> Result<(), ()> {
        let string = input!().fmt();
        *buffer = string.try_into()?;
        Ok(())
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
        "Commands:".bright_yellow().to_string(),
        "".to_string(),
        cmd_desc!("help", "displays this menu"),
        cmd_desc!("north", "moves the player north"),
        cmd_desc!("south", "moves the player south"),
        cmd_desc!("east", "moves the player east"),
        cmd_desc!("west", "moves the player west"),
        cmd_desc!("location", "displays your current location"),
        cmd_desc!("objects", "displays all objects in your current location"),
        cmd_desc!("interact", "interacts with an object in your current location"),
        cmd_desc!("save", format!("{} {}", "saves the game".cyan(), "(UNIMPLEMENTED)".red())),
        cmd_desc!("quit", "quits the game"),
    ];
    pall![lines];
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

pub enum BattleCommand {
    Attack,
    Run,
    Health,
    Options,
}

impl TryFrom<String> for BattleCommand {
    type Error = ();
    fn try_from(val: String) -> Result<Self, Self::Error> {
        use BattleCommand as BC;
        match val.as_str() {
            "attack" => Ok(BC::Attack),
            "run" => Ok(BC::Run),
            "health" => Ok(BC::Health),
            "options" | "help" => Ok(BC::Options),
            _ => Err(())
        }
    }
}

impl BattleCommand {
    pub fn from_user(prompt: &str) -> BattleCommand {
        let string = input!(prompt).fmt();
        match string.try_into() {
            Ok(v) => v,
            Err(_) => {
                println!("Please enter a valid command.");
                println!("Use {} to see commands.", "options".bright_yellow());
                println!();
                BattleCommand::from_user(prompt)
            }
        }
    }
}

pub fn process_battle(player: &mut Player, enemy: &mut Enemy) {
    use BattleCommand as BC;
    println!("You entered battle with {}.", enemy.name);
    // Note: a continue means to ask for another command
    // WITHOUT doing the damage calculation.
    loop {
        let cmd = BattleCommand::from_user(&("What action do you want to take? ".green()));
        match cmd {
            BC::Run => {
                if !enemy.can_run {
                    println!("You cannot run from this battle.");
                    continue;
                }
                const RUN_CHANCE: f32 = 0.80;
                if fastrand::f32() < RUN_CHANCE {
                    println!("You successfully ran away from the battle!");
                    return;
                } else {
                    println!("You failed to run away from the battle!");
                }
            },
            BC::Attack => {
                println!("You dealt {} damage!", player.weapon.damage());
                enemy.health =
                    enemy.health.checked_sub(player.weapon.damage())
                    .unwrap_or(0);
                println!("The enemy has {} health remaining.", enemy.health);
                sleep(Duration::from_millis(500));
            },
            BC::Health => {
                println!("Your health is: {}/{}", player.health, player.max_health);
                continue;
            },
            BC::Options => {
                let lines = [
                    "You can:".green().to_string(),
                    cmd_desc!("attack", "attack the enemy"),
                    cmd_desc!("run", "run away from the battle"),
                    cmd_desc!("health", "check your current health"),
                    cmd_desc!("options", "display this menu"),
                ];
                pall![lines];
                continue;
            }
        }
        // check enemy health
        if enemy.health <= 0 {
            println!("You defeated the {}!", enemy.name);
            println!("You gained {} xp!", enemy.xp);
            player.xp = player.xp.checked_add(enemy.xp)
                .unwrap_or(u16::MAX);
            println!("You now have {} xp!", player.xp);
        }
        // do enemy attack
        let attack: u16 = (enemy.damage)();
        println!("The {} dealt {} damage!", enemy.name, attack);
        player.health = player.health.checked_sub(attack).unwrap_or(0);
        println!("You have {}/{} health remaining.", player.health, player.max_health);
        // check player health
        if player.health <= 0 {
            // TODO: make the dots appear one by one
            println!("You died...");
            let death = "GAME OVER".red();
            println!("{death}");
            return;
        }
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
