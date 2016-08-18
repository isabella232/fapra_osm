extern crate osmpbfreader;
extern crate time;
#[macro_use] extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate rustc_serialize;
extern crate flate2;
extern crate bincode;
extern crate urlencoded;
extern crate ordered_float;

use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, BufReader};

use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use bincode::rustc_serialize::{encode_into, decode_from};

mod parser;
mod server;
mod data;
mod tsm;

const STATE_FILE_NAME: &'static str = "state.bin.gz";

fn main() {
	//	let data = match fs::metadata(STATE_FILE_NAME) {
	//		Ok(metadata) => {
	//			if metadata.is_file() {
	//				read_from_disk()
	//			} else {
	//				perform_parse()
	//			}
	//		},
	//		Err(_) => perform_parse(),
	//	};

	//server::start(data);

	tsm::startRDS();
}

fn perform_parse() -> data::State {
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

fn write_to_disk(data: &data::State) {
	println!("Writing state data to file {}.. ", STATE_FILE_NAME);
	let writer = BufWriter::new(File::create(STATE_FILE_NAME).unwrap());
	let mut encoder = ZlibEncoder::new(writer, Compression::Best);
	encode_into(&data, &mut encoder, bincode::SizeLimit::Infinite).unwrap();
	println!("Writing state data to file {}.. OK", STATE_FILE_NAME);
}

fn read_from_disk() -> data::State {
	println!("Reading state data from file {}.. ", STATE_FILE_NAME);
	let reader = BufReader::new(File::open(STATE_FILE_NAME).unwrap());
	let mut decoder = ZlibDecoder::new(reader);
	let decoded: data::State = decode_from(&mut decoder, bincode::SizeLimit::Infinite).unwrap();
	println!("Reading state data from file {}.. OK", STATE_FILE_NAME);
	return decoded;
}

fn build_dummy_data() -> data::State {
	parser::build_dummy_data()
}