mod game;
mod bot_runner;
mod games;
mod match_runner;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate rand;

use std::error::Error;
use std::io::{Write, Read};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::fs::File;

use game::*;
use bot_runner::*;
use match_runner::*;

//use games::HigherLower as Rules;

// Load the config and start the game.
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        let msg = format!("Expected 1 argument (config file). {} given.", args.len() - 1).to_owned();
        println!("{}", msg);
        std::process::exit(1)
    }

    let game_config: MatchDescription = match parse_config(Path::new(&args[1])) {
        Ok(config) => config,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1)
        }
    };
    //MatchRunner::<Rules>::run((), &game_config.players);
}

// Parse a config passed to the program as an command-line argument.
// Return the parsed config.
pub fn parse_config(path: &Path) -> Result<MatchDescription, Box<Error>> {
    println!("Opening config {}", path.to_str().unwrap());
    let mut file = File::open(path)?;

    println!("Reading contents");
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Parsing config");
    let config: MatchDescription = serde_json::from_str(&contents)?;

    println!("Config parsed succesfully");
    Ok(config)
}
