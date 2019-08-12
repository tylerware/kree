use crate::{keys, client};

use serde_yaml;
use serde_yaml::Value;

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

pub struct Trigger {
    class: String
}

pub struct Kree {
    pub global_keymap: Vec<(keys::KeyCombo, keys::Command)>,
    pub conditional_keymaps: Vec<(Trigger, Vec<(keys::KeyCombo, keys::Command)>)>,
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

                        let to_spawn: String = serde_yaml::from_value(to_spawn).unwrap();
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
                        self.client.register_keymap(Self::parse_keymap(&serde_yaml::from_value(keymap).unwrap()).unwrap(), true);
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
        println!("Config {:?}", config);

        let global_keymap = Self::parse_keymap(&serde_yaml::from_value(config.get("global").unwrap().clone()).unwrap()).unwrap();


        let mut conditional_keymaps: Vec<(Trigger, Vec<(keys::KeyCombo, keys::Command)>)> = vec![];
        if config.contains_key("conditional") {
            let raw_conditional_keymaps: Vec<HashMap<String, Value>> = serde_yaml::from_value(config.get("conditional").unwrap().clone()).unwrap();
            conditional_keymaps = raw_conditional_keymaps.into_iter()
                    .filter(|conditional_keymap| {
                        for mapping_field in vec!["trigger", "map"] {
                            if !conditional_keymap.contains_key(mapping_field) {
                                println!("Conditional keymap is missing field '{}'", mapping_field);
                                return false;
                            }

                            if !conditional_keymap.get(mapping_field).unwrap().is_mapping() {
                                println!("Conditional keymap field, '{}', must be a mapping", mapping_field);
                                return false;
                            }
                        }

                        true
                    })
                    .map(|conditional_keymap| {
                        let trigger_value: HashMap<String, String> = serde_yaml::from_value(conditional_keymap.get("trigger").unwrap().clone()).unwrap();
                        let trigger = Self::parse_trigger(&trigger_value).unwrap();

                        let keymap_value: HashMap<String, Value> = serde_yaml::from_value(conditional_keymap.get("map").unwrap().clone()).unwrap();
                        let keymap = Self::parse_keymap(&keymap_value).unwrap();

                        println!("Conditional keymap: {:?} ---> {:?}", trigger_value, keymap_value);

                        (trigger, keymap)
                    })
                    .collect();
        };

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

    fn parse_keymap(raw_keymap: &HashMap<String, Value>) -> Result<Vec<(keys::KeyCombo, keys::Command)>, serde_yaml::Error> {
        let keymap: Vec<(keys::KeyCombo, keys::Command)> = raw_keymap.clone()
            .into_iter()
            .filter(|(_key_chord, val)| val.is_string() || val.is_mapping())
            .map(|(key_chord, val)| {
                let (key, mods) = keys::parse_key_chord(key_chord);
                let mut command: Command;
                if val.is_string() {
                    command = Command::Spawn(val);
                }
                else if val.is_mapping() {
                    command = Command::Mapping(val);
                }
                else {
                    command = Command::Noop();
                }
                (KeyCombo::new(Event::KeyDown, &mods, key), command)
            })
            .collect();

        Ok(keymap)
    }

    fn parse_trigger(raw_trigger: &HashMap<String, String>) -> Result<Trigger, serde_yaml::Error> {
        let mut trigger = Trigger {
            class: "".to_string()
        };

        if let Some(class) = raw_trigger.get("class") {
            trigger.class = class.to_string();
        }

        Ok(trigger)
    }

}
