use time::PreciseTime;
use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use std::collections::HashSet;
use std::f64;

use osmpbfreader::OsmObj;
use osmpbfreader::OsmPbfReader;

#[derive(Debug, Clone)]
struct ParsedEdge {
	id_from: i64,
	id_to: i64,
	length: f64,
	constraints: u8,
	speed: f64
}

struct ParseData {
	// used node-ids
	nodes_used: HashSet<i64>,
	// "useful" ways
	filtered_ways: HashMap<i64, WayConstraints>,
	// relevant nodes and their position
	nodes: HashMap<i64, ::data::Position>,
	// edges
	edges: Vec<ParsedEdge>
}

#[derive(Debug, Clone)]
struct WayConstraints {
	access: u8,
	speed: f64
}

struct WayDefaults {
	lookup: HashMap<&'static str, WayConstraints>,
	default: WayConstraints
}

enum OneWay {
	NO,
	YES,
	REVERSE
}


pub fn read_file(filename: &OsString) -> ::data::State {
	println!("will read file: {:?}", &filename);

	let mut parse_result = ParseData { nodes_used: HashSet::new(), filtered_ways: HashMap::new(), nodes: HashMap::new(), edges: Vec::new() };

	let start_p1 = PreciseTime::now();
	first_parse(&filename, &mut parse_result);
	let end_p1 = PreciseTime::now();

	println!("P1 | ways:  {}", parse_result.filtered_ways.len());
	println!("P1 | nodes_used: {}", parse_result.nodes_used.len());
	println!("P1 | edges: {}", parse_result.edges.len());
	println!("P1 | nodes: {}", parse_result.nodes.len());
	println!("P1 | duration: {}", start_p1.to(end_p1));

	let start_p2 = PreciseTime::now();
	second_parse(&filename, &mut parse_result);
	let end_p2 = PreciseTime::now();

	println!("P2 | ways:  {}", parse_result.filtered_ways.len());
	println!("P2 | nodes_used: {}", parse_result.nodes_used.len());
	println!("P2 | edges: {}", parse_result.edges.len());
	println!("P2 | nodes: {}", parse_result.nodes.len());
	println!("P2 | duration: {}", start_p2.to(end_p2));

	let start_p3 = PreciseTime::now();
	third_parse(&filename, &mut parse_result);
	let end_p3 = PreciseTime::now();

	println!("P3 | ways:  {}", parse_result.filtered_ways.len());
	println!("P3 | nodes_used: {}", parse_result.nodes_used.len());
	println!("P3 | edges: {}", parse_result.edges.len());
	println!("P3 | nodes: {}", parse_result.nodes.len());
	println!("P3 | duration: {}", start_p3.to(end_p3));

	let start_b = PreciseTime::now();
	let routing_data = build_routing_data(parse_result);
	let end_b = PreciseTime::now();

	println!("B  | edges:  {}", routing_data.internal_edges.len());
	println!("B  | nodes:  {}", routing_data.internal_nodes.len());
	println!("B  | offset:  {}", routing_data.internal_offset.len());
	println!("B  | osm_nodes:  {}", routing_data.osm_nodes.len());
	println!("B  | duration: {}", start_b.to(end_b));

	let start_g = PreciseTime::now();
	let grid = build_grid(&routing_data);
	let end_g = PreciseTime::now();

	println!("G  | bounds: {:?}", grid.bbox);
	println!("G  | bin_cnt_lat: {}", grid.bin_count_lat);
	println!("G  | bin_cnt_lon: {}", grid.bin_count_lon);
	println!("G  | duration: {}", start_g.to(end_g));

	return ::data::State { routing_data: routing_data, grid: grid };
}

#[test]
#[ignore]
fn test_routing_data_gen() {
	let routing_data = build_dummy_data();

	println!("NODES: {:?}", routing_data.internal_nodes);
	println!("EDGES: {:?}", routing_data.internal_edges);
	println!("OFFSET: {:?}", routing_data.internal_offset);

	assert_eq!(routing_data.internal_offset, vec![0, 2, 2, 4, 5]);
}

pub fn build_dummy_data() -> ::data::State {
	let edge_vec = vec![ParsedEdge{id_from: 5000, id_to: 5001, length: 1.0, constraints: ::data::FLAG_CAR, speed: 13.89},
                        ParsedEdge{id_from: 5000, id_to: 5002, length: 10.0, constraints: ::data::FLAG_CAR, speed: 13.89},
                        ParsedEdge{id_from: 5002, id_to: 5001, length: 100.0, constraints: ::data::FLAG_CAR, speed: 13.89},
                        ParsedEdge{id_from: 5002, id_to: 5003, length: 1000.0, constraints: ::data::FLAG_CAR, speed: 13.89},
                        ParsedEdge{id_from: 5003, id_to: 5000, length: 10000.0, constraints: ::data::FLAG_CAR, speed: 13.89},
                        ParsedEdge{id_from: 5003, id_to: 5004, length: 100000.0, constraints: ::data::FLAG_CAR, speed: 13.89},
];


	let mut nodes_map = HashMap::new();

	nodes_map.insert(5000, ::data::Position { lat: 0.0, lon: 0.0 });
	nodes_map.insert(5001, ::data::Position { lat: 0.0, lon: 0.0 });
	nodes_map.insert(5002, ::data::Position { lat: 0.0, lon: 0.0 });
	nodes_map.insert(5003, ::data::Position { lat: 0.0, lon: 0.0 });
	nodes_map.insert(5004, ::data::Position { lat: 0.0, lon: 0.0 });

	let parse_result = ParseData { nodes: nodes_map, edges: edge_vec, filtered_ways: HashMap::new(), nodes_used: HashSet::new() };

	let routing_data = build_routing_data(parse_result);
	let grid = build_grid(&routing_data);
	::data::State { routing_data: routing_data, grid: grid }
}


fn first_parse(filename: &OsString, parse_result: &mut ParseData) {
	let pbf_file = File::open(&Path::new(filename)).unwrap();

	let filters = init_filter_lists();

	let mut pbf = OsmPbfReader::new(pbf_file);

	for obj in pbf.iter() {
		match obj {
			OsmObj::Way(way) => {
				if let Some(constraints) = filter_way(&way, &filters) {
					for node in way.nodes {
						parse_result.nodes_used.insert(node);
					}
					parse_result.filtered_ways.insert(way.id, constraints);
				}
			}
			_ => {}
		}
	}
}

fn second_parse(filename: &OsString, parse_result: &mut ParseData) {
	let pbf_file = File::open(&Path::new(filename)).unwrap();
	let mut pbf = OsmPbfReader::new(pbf_file);

	for obj in pbf.iter() {
		match obj {
			OsmObj::Node(node) => {
				if parse_result.nodes_used.remove(&node.id) {
					parse_result.nodes.insert(node.id, ::data::Position { lat: node.lat, lon: node.lon });
				}
			}
			_ => {}
		}
	}
}

fn third_parse(filename: &OsString, parse_result: &mut ParseData) {
	let pbf_file = File::open(&Path::new(filename)).unwrap();
	let mut pbf = OsmPbfReader::new(pbf_file);

	for obj in pbf.iter() {
		match obj {
			OsmObj::Way(way) => {
				let one_way = check_oneway(&way);
				if let Some(constraints) = parse_result.filtered_ways.remove(&way.id) {
					for node_pair in way.nodes.windows(2) {
						if let (Some(from), Some(to)) = (node_pair.first(), node_pair.last()) {
							if let (Some(from_node), Some(to_node)) = (parse_result.nodes.get(from), parse_result.nodes.get(to)) {
								let edge_length = from_node.distance(&to_node);
								let edge = ParsedEdge { id_from: *from, id_to: *to, length: edge_length, constraints: constraints.access, speed: constraints.speed };
								let edge_reverse = ParsedEdge { id_from: *to, id_to: *from, length: edge_length, constraints: constraints.access, speed: constraints.speed };

								match one_way {
									OneWay::NO => {
										parse_result.edges.push(edge);
										parse_result.edges.push(edge_reverse)
									},
									OneWay::YES => { parse_result.edges.push(edge) },
									OneWay::REVERSE => { parse_result.edges.push(edge_reverse) },
								}
							}
						}
					}
				}
			}
			_ => {}
		}
	}
}

fn build_routing_data(mut parse_result: ParseData) -> ::data::RoutingData {
	let mut routing_data = ::data::RoutingData { osm_nodes: HashMap::new(), internal_nodes: Vec::new(), internal_edges: Vec::new(), internal_offset: vec![usize::max_value(); parse_result.nodes.len()] };

	parse_result.edges.sort_by(|a, b| b.id_from.cmp(&a.id_from));

	for node in parse_result.nodes.keys() {
		routing_data.internal_nodes.push(node.clone());
	}

	routing_data.internal_nodes.sort();


	for (i, node) in routing_data.internal_nodes.iter().enumerate() {
		if let Some(pos) = parse_result.nodes.remove(node) {
			routing_data.osm_nodes.insert(node.clone(), ::data::OsmNode { position: pos, internal_id: i });
		}
	}

	for (i, node) in routing_data.internal_nodes.iter().enumerate() {
		if let Some(edge) = parse_result.edges.last() {
			if edge.id_from == *node {
				routing_data.internal_offset[i] = routing_data.internal_edges.len();
			}
		}

		loop {
			if let Some(edge) = parse_result.edges.last() {
				if edge.id_from != *node {
					break;
				}
			} else {
				break;
			}
			if let Some(edge) = parse_result.edges.pop() {
				let internal_target = routing_data.osm_nodes.get(&edge.id_to).unwrap().internal_id;

				routing_data.internal_edges.push(::data::RoutingEdge { target: internal_target, length: edge.length, constraints: edge.constraints, speed: edge.speed });
			} else {
				break;
			}
		}
	}

	let mut current_offset = routing_data.internal_edges.len() - 1;

	for offset in &mut routing_data.internal_offset.iter_mut().rev() {
		if *offset == usize::max_value() {
			*offset = current_offset;
		} else {
			current_offset = *offset;
		}
	}

	return routing_data;
}

fn build_grid(routing_data: &::data::RoutingData) -> ::data::Grid {
	let mut bbox = calculate_bounding_box(&routing_data);

	let grid_padding = 0.001;

	bbox.min_lat -= grid_padding;
	bbox.min_lon -= grid_padding;
	bbox.max_lat += grid_padding;
	bbox.max_lon += grid_padding;

	let bin_count = routing_data.osm_nodes.len() / 1024;

	let cnt_lat = (bin_count / ((bbox.max_lat - bbox.min_lat) as usize)) / 2;
	let cnt_lon = (bin_count / ((bbox.max_lon - bbox.min_lon) as usize)) / 2;

	let mut grid = ::data::Grid { bbox: bbox, bins: vec![::data::Bin{nodes: Vec::new()};cnt_lat * cnt_lon], bin_count_lat: cnt_lat, bin_count_lon: cnt_lon };

	for (id, node) in &routing_data.osm_nodes {
		let (lat_bin, lon_bin) = grid.calc_bin_index(&node.position);

		let index = grid.calc_bin_position(lat_bin, lon_bin);

		if let Some(bin) = grid.bins.get_mut(index) {
			bin.nodes.push(*id);
		} else {
			println!("error inserting node, {} is out of bounds of bin array {}", index, bin_count);
		}
	}

	grid
}

fn calculate_bounding_box(routing_data: &::data::RoutingData) -> ::data::BoundingBox {
	let mut bbox = ::data::BoundingBox { max_lat: f64::NEG_INFINITY, max_lon: f64::NEG_INFINITY, min_lat: f64::INFINITY, min_lon: f64::INFINITY };

	for (_, node) in &routing_data.osm_nodes {
		if node.position.lat > bbox.max_lat {
			bbox.max_lat = node.position.lat;
		}
		if node.position.lon > bbox.max_lon {
			bbox.max_lon = node.position.lon;
		}
		if node.position.lat < bbox.min_lat {
			bbox.min_lat = node.position.lat;
		}
		if node.position.lon < bbox.min_lon {
			bbox.min_lon = node.position.lon;
		}
	}
	bbox
}


fn init_filter_lists() -> WayDefaults {
	let mut defaults = WayDefaults { default: WayConstraints { speed: 1.0, access: ::data::FLAG_CAR | ::data::FLAG_WALK | ::data::FLAG_BIKE }, lookup: HashMap::new(), ignore: HashSet::new() };
	// @formatter:off
    defaults.lookup.insert("primary", 			WayConstraints { speed: 130.0,  access:  ::data::FLAG_CAR });
    defaults.lookup.insert("trunk", 			WayConstraints { speed: 120.0,  access:  ::data::FLAG_CAR });
    defaults.lookup.insert("motorway", 			WayConstraints { speed: 100.0,  access:  ::data::FLAG_CAR });
    defaults.lookup.insert("secondary", 		WayConstraints { speed: 100.0,  access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("tertiary", 			WayConstraints { speed:  80.0,	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("unclassified", 		WayConstraints { speed:  50.0,	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("residential", 		WayConstraints { speed:  30.0,	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("service", 			WayConstraints { speed:   5.0,	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("motorway_link", 	WayConstraints { speed:  80.0,	access:  ::data::FLAG_CAR });
    defaults.lookup.insert("trunk_link", 		WayConstraints { speed:  80.0,	access:  ::data::FLAG_CAR });
    defaults.lookup.insert("primary_link", 		WayConstraints { speed:  80.0,	access:  ::data::FLAG_CAR });
    defaults.lookup.insert("secondary_link", 	WayConstraints { speed:  80.0,	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("tertiary_link", 	WayConstraints { speed:  8.00,	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("living_street", 	WayConstraints { speed:   5.0,  access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("pedestrian", 		WayConstraints { speed:   5.0,  access:  ::data::FLAG_WALK });
    defaults.lookup.insert("track", 			WayConstraints { speed:  10.0,  access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("bus_guide_way", 	WayConstraints { speed:   5.0,  access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("raceway", 			WayConstraints { speed: 300.0,  access:  ::data::FLAG_CAR });
    defaults.lookup.insert("road", 				WayConstraints { speed:   5.0, 	access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("footway", 			WayConstraints { speed:   5.0,  access:  ::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("bridleway",			WayConstraints { speed:   5.0,  access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("steps", 			WayConstraints { speed:   5.0,  access:  ::data::FLAG_WALK });
    defaults.lookup.insert("path", 				WayConstraints { speed:   5.0,  access:  ::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("cycleway", 			WayConstraints { speed:   5.0, 	access:  ::data::FLAG_BIKE });
    defaults.lookup.insert("bus_stop", 			WayConstraints { speed:   5.0,  access:  ::data::FLAG_CAR|::data::FLAG_BIKE|::data::FLAG_WALK });
    defaults.lookup.insert("platform", 			WayConstraints { speed:   5.0, 	access:  ::data::FLAG_WALK });
    // @formatter:on

	return defaults;
}

fn filter_way(way: &::osmpbfreader::Way, defaults: &WayDefaults) -> Option<WayConstraints> {
	if let Some(value) = way.tags.get("highway") {
		if let Some(default_constraints) = defaults.lookup.get(&value.as_str()) {
			let mut constraints = default_constraints.clone();

			if bike_denied(&way) {
				constraints.access &= ::data::FLAG_BIKE ^ 0xff;
			}
			if walk_denied(&way) {
				constraints.access &= ::data::FLAG_WALK ^ 0xff;
			}

			check_speed(&mut constraints, &way);

			return Some(constraints);
		} else {
			return None;
		}
	}
	return None;
}

fn check_oneway(way: &::osmpbfreader::Way) -> OneWay {
	let highway_1 = check_key_and_value(way, "highway", "motorway");
	let highway_2 = check_key_and_value(way, "highway", "motorway_link");
	let roundabout = check_key_and_value(way, "junction", "roundabout");

	if highway_1 || highway_2 || roundabout {
		return OneWay::YES;
	}

	let yes = vec!["true", "1", "yes"];
	let reverse = vec!["-1", "reverse"];

	if let Some(entry) = way.tags.get("oneway") {
		if yes.contains(&entry.as_str()) {
			return OneWay::YES;
		} else if reverse.contains(&entry.as_str()) {
			return OneWay::REVERSE;
		}
	}

	return OneWay::NO;
}

fn bike_denied(way: &::osmpbfreader::Way) -> bool {
	let motorroad = check_key_and_value(way, "motorroad", "true");
	let no_bike = check_key_and_value(way, "bicycle", "false");

	return motorroad || no_bike;
}

fn walk_denied(way: &::osmpbfreader::Way) -> bool {
	let motorroad = check_key_and_value(way, "motorroad", "true");
	let no_walk = check_key_and_value(way, "foot", "false");

	return motorroad || no_walk;
}

fn check_key_and_value(way: &::osmpbfreader::Way, key: &str, value: &str) -> bool {
	if let Some(entry) = way.tags.get(key) {
		return entry == value;
	}
	return false;
}

fn check_speed(data: &mut WayConstraints, way: &::osmpbfreader::Way) {
	if let Some(full_string) = way.tags.get("maxspeed") {
		let mut elements = full_string.split_whitespace();
		if let Some(speed_string) = elements.next() {
			if let Ok(speed) = speed_string.parse::<u32> () {
				let fspeed = speed as f64;
				if fspeed > 0.0 {
					data.speed = fspeed;
					if let Some(unit_string) = elements.next() {
						if unit_string.starts_with("mph") {
							data.speed = 1.6 * data.speed;
						}
					}
				} else {
					println!("speed is 0.0 for {:?}", way);
				}
			}
		}
	}

	data.speed = data.speed / 3.6;
}