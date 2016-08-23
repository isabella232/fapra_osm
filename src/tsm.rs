use std::process::{Command, Stdio};
use std::io::{BufRead, Write, BufReader};
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::path::Path;

use osmpbfreader::OsmObj;
use osmpbfreader::Tags;
use osmpbfreader::OsmPbfReader;

pub fn startRDS() {
	readRDSTags();
}

pub fn readRDSQueryInput() {
	let mut rdsd_child = Command::new("rdsd").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("rdsd command failed to start");
	let mut rdsquery_child = Command::new("rdsquery").arg("-s").arg("localhost").arg("-c").arg("0").arg("-t").arg("tmc").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().expect("rdsquery command failed to start");

	let mut buf_reader = BufReader::new(rdsquery_child.stdout.as_mut().unwrap());

	for line in buf_reader.lines() {
		if let Ok(content) = line {
			if content.starts_with("GS ") {
				println!("GS:: {}", content);
			} else if content.starts_with("GF ") {
				println!("GF:: {}", content);
			} else if content.starts_with("S ") {
				println!("S::  {}", content);
			}
		}
	}

	rdsd_child.kill();
}

pub fn readRDSTags() {
	let filename = OsString::from("/home/jan/Downloads/baden-wuerttemberg-latest.osm.pbf".to_string());
	let pbf_file = File::open(&Path::new(&filename)).unwrap();

	let mut pbf = OsmPbfReader::new(pbf_file);

	for obj in pbf.iter() {
		match obj {
			OsmObj::Node(node) => {
				printTMCTags(node.tags, "node", node.id);
			}
			OsmObj::Relation(rel) => {
				//printTMCTags(rel.tags, "rel", rel.id);
			}
			OsmObj::Way(way) => {
				printTMCTags(way.tags, "way", way.id);
			}
		}
	}
}

fn printTMCTags(tags: Tags, typ: &str, id: i64) {
	for (tag, val) in tags {
		if tag == "tmc" || tag == "TMC" {
			println!("{}->{} in {} {}", tag, val, typ, id);
		}
	}
}