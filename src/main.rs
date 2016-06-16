extern crate osmpbfreader;
extern crate time;
extern crate iron;
extern crate staticfile;
extern crate mount;

use std::ffi::OsString;

mod parser;
mod server;

fn main() {
    //perform_parse();
    perform_server();
}

fn perform_server() {
    server::start();
}

fn perform_parse() {
    let default_file = OsString::from("/home/zsdn/baden-wuerttemberg-latest.osm.pbf".to_string());
    //let default_file = OsString::from("/home/zsdn/germany-latest.osm.pbf".to_string());

    let args: Vec<OsString> = std::env::args_os().collect();
    match args.len() {
        1 => {
            parser::read_file(&default_file);
        }
        2 => {
            parser::read_file(&args[1]);
        }
        _ => {},
    };
}