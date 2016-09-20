use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

//fn run_TMC_thread(tmc_state: RwLock<::data::TMCState>, data: ::data::State) {
pub fn run_tmc_thread(tmc_arc: Arc<RwLock<::data::TMCState>>, data_arc: Arc<::data::State>) {
	insert_dummy_events(tmc_arc, data_arc);
	//run_rdsd_loop(tmc_arc, data_arc);
}

fn insert_dummy_events(tmc_arc: Arc<RwLock<::data::TMCState>>, data: Arc<::data::State>) {
	let raw_event = ::data::TMCRawEvent { loc: 11602, dir: true, event: 701, ext: 2 };
	let raw_event2 = ::data::TMCRawEvent { loc: 11593, dir: true, event: 702, ext: 2 };

	let mut state = tmc_arc.write().unwrap();

	handle_event(raw_event, &mut state, &data);
	handle_event(raw_event2, &mut state, &data);

	println!("{}", state.current_tmc_events.len());
	println!("{}", state.current_edge_events.len());
}

fn handle_event(raw_event: ::data::TMCRawEvent, state: &mut ::data::TMCState, data: &::data::State) {
	let tmc_ids = build_tmc_range_set(&raw_event, data);

	let slowdown = lookup_slowdown(&raw_event.event);
	let desc = lookup_desc(&raw_event.event);

	let key = ::data::TMCKey { dir: raw_event.dir, loc: raw_event.loc, event: raw_event.event };
	let mut value = ::data::TMCEvent { ext: raw_event.event, desc: desc, edges: HashSet::new(), slowdown: slowdown, timeout: 1000 * 10 };

	for tmc_id in tmc_ids {
		if let Some(edges) = data.routing_data.tmc_mapping.get(&tmc_id) {
			for edge in edges {
				value.edges.insert(*edge);
				state.current_edge_events.insert(*edge, slowdown);
			}
		}
	}

	if !value.edges.is_empty() {
		state.current_tmc_events.insert(key, value);
	} else {
		println!("no edges found for loc id {}", raw_event.loc);
	}
}

fn lookup_slowdown(event: &u32) -> f64 {
	//TODO
	return 40.0;
}

fn lookup_desc(event: &u32) -> String {
	// TODO
	return format!("event {}", event);
}

fn build_tmc_range_set(raw_event: &::data::TMCRawEvent, data: &::data::State) -> Vec<u32> {
	let mut result = Vec::new();
	result.push(raw_event.loc);

	if raw_event.ext == 0 {
		return result;
	}

	let mut curr_id = raw_event.loc;
	let mut curr_dist = raw_event.ext;

	while let Some(next_id) = data.routing_data.tmc_next.get(&(curr_id, raw_event.dir)) {
		result.push(*next_id);

		curr_id = *next_id;
		curr_dist -= 1;

		if curr_dist == 0 {
			break;
		}
	}

	println!("range_set for loc {} {} {} = {:?}", raw_event.loc, raw_event.dir, raw_event.ext, result);

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
