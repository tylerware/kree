extern crate clap;
use clap::{App, Arg};

mod kree;
mod keys;
mod client;

use kree::Kree;
fn main() {
    let matches = App::new("kree")
        .version("0.1.0")
        .about("A tree style keymapper.")
        .author("Tyler Ware")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets another config file to use. Does not override $HOME/.kree.yaml. \nThere can be multiple. If a mapped key conflicts with another config the last config is used.")
             .multiple(true)
             .takes_value(true))
        .get_matches();


    let mut configs: Vec<String>;
    match matches.values_of_lossy("config") {
        Some(values) => configs = values,
        None => configs = vec![]
    }

    Kree::start(configs);
}
