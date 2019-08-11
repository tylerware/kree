use crate::{keys, client};

use serde_yaml;
use serde_yaml::{Value, Mapping};

use dirs;
use std::path::{Path};

use std::fs::File;
use std::io::Read;
use std::process;
use std::collections::HashMap;

use keys::{Event, KeyCombo, Command};
use keys::Command::*;

use client::{Client};
use client::Event::*;

pub struct Kree {
    pub global_keymap: Vec<(keys::KeyCombo, keys::Command)>,
    pub conditional_keymaps: Vec<Vec<(keys::KeyCombo, keys::Command)>>,
    pub client: Client,
}

impl Kree {
    pub fn start() {
        println!("Starting kree...");

        let mut kree = Self::new_from_config();
        kree.listen();

    }

    pub fn listen(&mut self) {
        loop {
            match self.client.poll() {
                Command(cmd) => match cmd {
                    Spawn(to_spawn) => {
                        // Root keymap
                        self.client.register_keymap(self.global_keymap.clone(), false);

                        let to_spawn = String::from(to_spawn);
                        let mut to_spawn_split = to_spawn.split_whitespace();

                        let executable = to_spawn_split.nth(0).unwrap();
                        let params = to_spawn_split.collect::<Vec<_>>();

                        match process::Command::new(executable)
                            .args(&params)
                            .spawn() {
                                Ok(_) => println!("Spawning: {}", to_spawn),
                                Err(error) => println!("Failed to spawn: {:?}", error),
                            }

                    },
                    Mapping(keymap) => {
                        println!("Handle keymap: {:?}", keymap);
                        self.client.register_keymap(Self::parse_keymap(&keymap).unwrap(), true);
                    }
                    Noop() => {
                        println!("No operation");
                        // Root keymap
                        self.client.register_keymap(self.global_keymap.clone(), false);
                    }
                }
            }
        }
    }

    pub fn new_from_config() -> Self {
        let config = Self::get_config().unwrap();

        let global_keymap = Self::parse_keymap(config.get("global").unwrap().as_mapping().unwrap()).unwrap();
        let conditional_keymaps: Vec<Vec<(keys::KeyCombo, keys::Command)>> = vec![];
        let mut client = Client::open_connection();
        client.register_keymap(global_keymap.clone(), false);

        let instance = Self {
            global_keymap: global_keymap,
            conditional_keymaps: conditional_keymaps,
            client: client,
        };

        instance
    }

    fn get_config() -> Result<HashMap<String, Value>, ()> {
        let mut content = String::new();

        let config_file_path = Path::new(&dirs::home_dir().unwrap()).join(".kree.yaml");
        match File::open(config_file_path) {
            // The file is open (no error).
            Ok(mut file) => {
                // Read all the file content into a variable
                file.read_to_string(&mut content).unwrap();
            },
            // Error handling.
            Err(error) => {
                println!("Error opening file: {}", error);
            },
        }

        let yaml_doc: HashMap<String, Value> = serde_yaml::from_str(&content).unwrap();

        Ok(yaml_doc)
    }

    fn parse_keymap(raw_keymap: &Mapping) -> Result<Vec<(keys::KeyCombo, keys::Command)>, serde_yaml::Error> {
        let keymap: Vec<(keys::KeyCombo, keys::Command)> = raw_keymap.clone()
            .into_iter()
            .filter(|(_key_chord, val)| val.is_string() || val.is_mapping())
            .map(|(key_chord, val)| {
                let (key, mods) = keys::parse_key_chord(key_chord.as_str().unwrap().to_string());
                let mut command: Command;
                if val.is_string() {
                    command = Command::Spawn(val.as_str().unwrap().to_string());
                }
                else if val.is_mapping() {
                    command = Command::Mapping(val.as_mapping().unwrap().clone());
                }
                else {
                    command = Command::Noop();
                }
                (KeyCombo::new(Event::KeyDown, &mods, key), command)
            })
            .collect();

        Ok(keymap)
    }

}
