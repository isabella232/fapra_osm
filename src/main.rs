extern crate osmpbfreader;
extern crate time;
extern crate iron;
extern crate staticfile;
extern crate mount;

use std::ffi::OsString;

mod parser;
mod server;

fn main() {
    //let data = perform_parse();
    let data = build_dummy_data();
    perform_server(data);
}

fn perform_server(data: parser::RoutingData) {
    server::start(data);
}

fn perform_parse() -> parser::RoutingData {
    let default_file = OsString::from("/home/zsdn/baden-wuerttemberg-latest.osm.pbf".to_string());
    //let default_file = OsString::from("/home/zsdn/germany-latest.osm.pbf".to_string());

    let args: Vec<OsString> = std::env::args_os().collect();
    let data = match args.len() {
        1 => parser::read_file(&default_file),
        2 => parser::read_file(&args[1]),
        _ => build_dummy_data(),
    };
    return data;
}

fn build_dummy_data() -> parser::RoutingData {
    parser::build_dummy_data()
}