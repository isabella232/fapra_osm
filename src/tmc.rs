use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::RwLock;
use std::sync::Arc;

//fn run_TMC_thread(tmc_state: RwLock<::data::TMCState>, data: ::data::State) {
pub fn run_tmc_thread(tmc_arc: Arc<RwLock<::data::TMCState>>, data_arc: Arc<::data::State>) {
	insert_dummy_events(tmc_arc, data_arc);
	//run_rdsd_loop(tmc_arc, data_arc);
}

fn insert_dummy_events(tmc_arc: Arc<RwLock<::data::TMCState>>, data: Arc<::data::State>) {
	let mut state = tmc_arc.write().unwrap();

	let id = 11602;
	let dir = true;

	let tmc_ids = build_tmc_range_set(id, dir, 2, &data);

	for tmc_id in tmc_ids {
		if let Some(edges) = data.routing_data.tmc_mapping.get(&tmc_id) {
			for edge in edges {
				state.current_edge_events.insert(*edge, ::data::TMCEvent { desc: "kek".to_string(), slowdown: 0.24 });
			}
		}
	}
}

fn build_tmc_range_set(tmc_id: u32, dir: bool, dist: u8, data: &Arc<::data::State>) -> Vec<u32> {
	let mut result = Vec::new();
	result.push(tmc_id);

	if dist == 0 {
		return result;
	}

	let mut curr_id = tmc_id;
	let mut curr_dist = dist;

	while let Some(next_id) = data.routing_data.tmc_next.get(&(curr_id, dir)) {
		result.push(*next_id);

		curr_id = *next_id;
		curr_dist -= 1;

		if curr_dist == 0 {
			break;
		}
	}


	return result;
}

fn run_rdsd_loop(tmc_arc: Arc<RwLock<::data::TMCState>>, data_arc: Arc<::data::State>) {
	let mut rdsd_child = Command::new("rdsd").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("rdsd command failed to start");
	let mut rdsquery_child = Command::new("rdsquery").arg("-s").arg("localhost").arg("-c").arg("0").arg("-t").arg("tmc").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().expect("rdsquery command failed to start");

	let buf_reader = BufReader::new(rdsquery_child.stdout.as_mut().unwrap());

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
