use std::collections::HashMap;
use std::process::Child;

use game_types::{Player};

/*
 * A player written configuration for a single game, 
 * containing metadata about for example:
 *  - players
 *  - maximum step count
 *  - ...
 * TODO: Change name
 */
 #[derive(Serialize, Deserialize, Debug)]
pub struct GameConfigFormat {
    pub players: Vec<PlayerConfig>
}

/*
 * An easier to (programmatically) use configuration format.
 * TODO: Change name
 */
 #[derive(Debug)]
pub struct GameConfig {
    pub players: HashMap<Player, PlayerConfig>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerConfig {
    pub name: Player,
    pub start_command: StartCommand,
    pub args: Args
}

/*
 * A series of handles to running processes we need to keep tabs of.
 */
pub type BotHandles = HashMap<Player, BotHandle>;

pub type BotHandle = Child;

/*
 * A command executing the bot in question.
 * Ex: python
 */
pub type StartCommand = String;

/*
 * An array of arguments needing to be passed to the StartCommand/
 */
pub type Args = Vec<String>;

 // TODO: Implement things with warnings (non-blocking faulty moves)
 // TODO: Implement things with logging