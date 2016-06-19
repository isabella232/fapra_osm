extern crate osmpbfreader;
extern crate time;
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate rustc_serialize;
extern crate flate2;
extern crate bincode;
extern crate urlencoded;
extern crate ordered_float;

use std::ffi::OsString;
use std::fs::File;
use std::io::{BufWriter, BufReader};

use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use bincode::rustc_serialize::{encode_into, decode_from};

mod parser;
mod server;

fn main() {
    let parse_flag = false;

    let data = match parse_flag {
        true => perform_parse(),
        false => read_from_disk(),
    };

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
    write_to_disk(&data);
    return data;
}

fn write_to_disk(data: &parser::RoutingData) {
    let writer = BufWriter::new(File::create("graph.bin.gz").unwrap());
    let mut encoder = ZlibEncoder::new(writer, Compression::Best);
    encode_into(&data, &mut encoder, bincode::SizeLimit::Infinite).unwrap();
}

fn read_from_disk() -> parser::RoutingData {
    let reader = BufReader::new(File::open("graph.bin.gz").unwrap());
    let mut decoder = ZlibDecoder::new(reader);
    let decoded: parser::RoutingData = decode_from(&mut decoder, bincode::SizeLimit::Infinite).unwrap();
    return decoded;
}

fn build_dummy_data() -> parser::RoutingData {
    parser::build_dummy_data()
}