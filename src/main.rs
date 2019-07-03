#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use serde_yaml;
use serde_yaml::{Value, Mapping};

use std::fs::File;
use std::io::Read;
use std::process;
use std::collections::HashMap;

mod keys;
mod client;


use keys::{Event, KeyCombo, Command};
use client::{Client};

fn main() {
    println!("Starting kree...");
    let mut client = Client::open_connection();
    client.register_keybinds(keybinds().unwrap());

    loop {
        use client::Event::*;
        use keys::Command::*;

        match client.poll() {
            Command(cmd) => match cmd {
                Spawn(to_spawn) => {
                    // println!("Attempting to spawn: {}", to_spawn);
                    let to_spawn = String::from(to_spawn);
                    let mut to_spawn_split = to_spawn.split_whitespace();

                    let executable = to_spawn_split.nth(0).unwrap();
                    let params = to_spawn_split.collect::<Vec<_>>();

                    process::Command::new(executable)
                        .args(&params)
                        .spawn()
                        .expect("Failed to spawn cmd");
                },
                Mapping(keymap) => {
                    println!("Handle keymap: {:?}", keymap);
                    // 1. Unregister keybinds,
                    // 2. Register next keybinds (perhaps we need to capture whole keyboard?)
                    // 3. Reregister root to the root keymap if next key doesn't match or the match is a leaf node
                }
                Noop() => {
                    println!("No operation");
                }
            }
        }
    }
}

fn keybinds() -> Result<Vec<(keys::KeyCombo, keys::Command)>, serde_yaml::Error> {
    let mut content = String::new();
    match File::open("/home/tware/Projects/kree/kree.yaml") {
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
    parse_keymap(yaml_doc.get("mappings").unwrap().as_mapping().unwrap())
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
