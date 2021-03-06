use std::fs::File;
use std::io::Read;
use std::io;

use serde_json;

use planetwars::modules::pw_protocol;
use planetwars::modules::pw_rules::*;
use planetwars::controller::PlayerId;


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub map_file: String,
    pub max_turns: u64,
}

impl Config {
    pub fn create_game(&self, num_players: usize) -> PlanetWars {
        let players = (0..num_players)
            .map(|id| Player { id: PlayerId::new(id), alive: true })
            .collect();
        let planets = self.load_map(num_players);
        
        PlanetWars {
            players: players,
            planets: planets,
            expeditions: Vec::new(),
            expedition_num: 0,
            turn_num: 0,
            max_turns: self.max_turns,
        }
    }
    
    fn load_map(&self, num_players: usize) -> Vec<Planet> {
        let map = self.read_map().expect("[PLANET_WARS] reading map failed");
        
        return map.planets
            .into_iter()
            .enumerate()
            .map(|(num, planet)| {
            let mut fleets = Vec::new();
                let owner = planet.owner.and_then(|num| {
                    // subtract one to convert from player num to player id
                    let id = num as usize - 1;
                    // ignore players that are not in the game
                    if id < num_players {
                        Some(PlayerId::new(id))
                    } else { 
                        None
                    }
                });
            if planet.ship_count > 0 {
                fleets.push(Fleet {
                    owner: owner,
                    ship_count: planet.ship_count,
                });
            }
            return Planet {
                id: num,
                name: planet.name,
                x: planet.x,
                y: planet.y,
                fleets: fleets,
            };
        }).collect();
    }

    fn read_map(&self) -> io::Result<pw_protocol::Map> {
        let mut file = File::open(&self.map_file)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let map = serde_json::from_str(&buf)?;
        return Ok(map);
    }
}
