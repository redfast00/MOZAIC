mod bot_runner;
mod client_controller;
mod buffered_sender;
mod planetwars;

extern crate bytes;

extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;
#[macro_use]
extern crate futures;

extern crate serde;
extern crate serde_json;
extern crate erased_serde;
#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;


#[macro_use]
extern crate slog;
extern crate slog_json;

use std::error::Error;
use std::io::{Read};
use std::env;
use std::path::Path;
use std::fs::File;

use slog::Drain;
use std::sync::Mutex;
use tokio_core::reactor::Core;
use futures::sync::mpsc;

use bot_runner::*;

use client_controller::ClientController;
use planetwars::{Controller, Client};

// Load the config and start the game.
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Expected 1 argument (config file). {} given.", args.len() - 1);
        std::process::exit(1)
    }

    let match_description: MatchDescription = match parse_config(Path::new(&args[1])) {
        Ok(config) => config,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1)
        }
    };

    let mut reactor = Core::new().unwrap();

    let log_file = File::create("log.json").unwrap();

    let logger = slog::Logger::root( 
        Mutex::new(slog_json::Json::default(log_file)).map(slog::Fuse),
        o!()
    );

    let mut bots = spawn_bots(&reactor.handle(), &match_description.players);

    let (handle, chan) = mpsc::unbounded();


    let handles = match_description.players.iter().enumerate().map(|(num, desc)| {
        let bot_handle = bots.remove(&desc.name).unwrap();
        let controller = ClientController::new(
            num,
            bot_handle,
            handle.clone(),
            &logger);
        let ctrl_handle = controller.handle();
        reactor.handle().spawn(controller);

        Client {
            id: num,
            player_name: desc.name.clone(),
            handle: ctrl_handle,
        }
    }).collect();


    let controller = Controller::new(
        handles,
        chan,
        match_description.game_config,
        logger,
    );

    reactor.run(controller).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct MatchDescription {
    pub players: Vec<PlayerConfig>,
    pub game_config: planetwars::Config,
    pub log_file: Option<String>
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