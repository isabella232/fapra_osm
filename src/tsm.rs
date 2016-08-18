use std::process::{Command, Stdio};
use std::io::{BufRead, Write, BufReader};

pub fn startRDS() {
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