use std::path::Path;
use std::sync::Arc;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::f64;
use iron::prelude::*;
use iron::status;
use staticfile::Static;
use mount::Mount;
use ordered_float::OrderedFloat;
use urlencoded::UrlEncodedQuery;
use rustc_serialize::json;
use time::PreciseTime;
use std::sync::RwLock;
use std::thread;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct HeapEntry {
	node: usize,
	cost: f64,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct RoutingResult {
	duration: i64,
	route: Option<Route>
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct TMCResult {
	events: Vec<TMCResultEntry>
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct TMCResultEntry {
	event: String,
	edges: Vec<TMCEdge>
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct TMCEdge {
	from: [f64; 2],
	to: [f64; 2]
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct Route {
	distance: f64,
	time: f64,
	path: Vec<[f64; 2]>
}

#[derive(Debug, Clone)]
struct PredecessorInfo {
	node: usize,
	edge: usize
}

impl Ord for HeapEntry {
	fn cmp(&self, other: &HeapEntry) -> Ordering {
		OrderedFloat(other.cost).cmp(&OrderedFloat(self.cost))
	}
}

impl PartialOrd for HeapEntry {
	fn partial_cmp(&self, other: &HeapEntry) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for HeapEntry {}

impl PartialEq for HeapEntry {
	fn eq(&self, other: &HeapEntry) -> bool {
		return (self.node == other.node) & &(OrderedFloat(other.cost).eq(&OrderedFloat(self.cost)))
	}
}


pub fn start(data: ::data::State) {
	let tmc_state = RwLock::new(::data::TMCState { current_edge_events: HashMap::new(), current_tmc_events: HashMap::new() });

	let data_wrapped = Arc::new(data);
	let data_wrapped_2 = data_wrapped.clone();
	let data_wrapped_3 = data_wrapped.clone();
	let data_wrapped_4 = data_wrapped.clone();
	let data_wrapped_5 = data_wrapped.clone();

	let tmc_state_wrapped = Arc::new(tmc_state);
	let tmc_state_wrapped_2 = tmc_state_wrapped.clone();
	let tmc_state_wrapped_3 = tmc_state_wrapped.clone();

	let mut mount = Mount::new();

	mount.mount("/", Static::new(Path::new("web/")));
	mount.mount("/api/hello", move |r: &mut Request| get_hello(r, &data_wrapped));
	mount.mount("/api/graph", move |r: &mut Request| get_graph(r, &data_wrapped_2));
	mount.mount("/api/route", move |r: &mut Request| get_route(r, &data_wrapped_3, &tmc_state_wrapped));
	mount.mount("/api/tmc", move |r: &mut Request| get_tmc(r, &data_wrapped_4, &tmc_state_wrapped_2));

	println!("server running on http://localhost:8080/");

	start_tmc_listener_thread(tmc_state_wrapped_3, data_wrapped_5);

	Iron::new(mount).http("127.0.0.1:8080").unwrap();
}

fn get_hello(req: &mut Request, data: &::data::State) -> IronResult<Response> {
	println!("Running get_hello handler, URL path: {:?}", req.url.path);
	Ok(Response::with((status::Ok, format!("HI! nodes: {}, edges: {}", data.routing_data.internal_nodes.len(), data.routing_data.internal_edges.len()))))
}

fn get_tmc(req: &mut Request, data: &::data::State, tmc_state: &RwLock<::data::TMCState>) -> IronResult<Response> {
	println!("Running get_tmc handler");

	let mut result = TMCResult { events: Vec::new() };
	let tmc = tmc_state.read().unwrap();

	for (tmc_key, tmc_value) in &tmc.current_tmc_events {
		let mut res = TMCResultEntry { event: tmc_value.desc.clone(), edges: Vec::new() };

		for edge_id in &tmc_value.edges {
			let ref edge = data.routing_data.internal_edges[*edge_id];

			let osm_id_from = data.routing_data.internal_nodes[edge.source];
			let osm_id_to = data.routing_data.internal_nodes[edge.target];

			let ref pos_from = data.routing_data.osm_nodes.get(&osm_id_from).unwrap().position;
			let ref pos_to = data.routing_data.osm_nodes.get(&osm_id_to).unwrap().position;

			res.edges.push(TMCEdge { from: [pos_from.lat, pos_from.lon], to: [pos_to.lat, pos_to.lon] });
		}

		result.events.push(res);
	}

	Ok(Response::with((status::Ok, json::encode(&result).unwrap())))
}

fn get_graph(req: &mut Request, data: &::data::State) -> IronResult<Response> {
	println!("Running get_graph handler, URL path: {:?}", req.url.path);
	Ok(Response::with((status::Ok, format!("nodes: {}, edges: {}", data.routing_data.internal_nodes.len(), data.routing_data.internal_edges.len()))))
}

fn parse_position(raw: &str) -> Option<::data::Position> {
	let mut split = raw.split(",");

	if let (Some(lat_str), Some(lon_str)) = (split.next(), split.next()) {
		if let (Ok(lat), Ok(lon)) = (lat_str.parse::<f64>(), lon_str.parse::<f64>()) {
			return Some(::data::Position { lat: lat, lon: lon });
		}
	}

	None
}

fn get_route(req: &mut Request, data: &::data::State, tmc_state: &RwLock<::data::TMCState>) -> IronResult<Response> {
	if let Ok(ref query_map) = req.get_ref::<UrlEncodedQuery>() {
		let source_raw = query_map.get("source").and_then(|list| list.first()).and_then(|string| Some(string.as_str())).unwrap_or("49.51807644873301,10.689697265625");
		let target_raw = query_map.get("target").and_then(|list| list.first()).and_then(|string| Some(string.as_str())).unwrap_or("48.30877444352327,10.12939453125");
		let metric_raw = query_map.get("metric").and_then(|list| list.first()).and_then(|string| Some(string.as_str())).unwrap_or("time");
		let vehicle_raw = query_map.get("vehicle").and_then(|list| list.first()).and_then(|string| Some(string.as_str())).unwrap_or("car");
		let use_tmc_raw = query_map.get("tmc").and_then(|list| list.first()).and_then(|string| Some(string.as_str())).unwrap_or("false");

		let source_pos = parse_position(source_raw).unwrap_or(::data::Position { lat: 49.51807644873301, lon: 10.689697265625 });
		let target_pos = parse_position(target_raw).unwrap_or(::data::Position { lat: 8.30877444352327, lon: 10.12939453125 });

		let source = data.grid.find_closest_node(&source_pos, &data.routing_data);
		let target = data.grid.find_closest_node(&target_pos, &data.routing_data);

		let use_tmc = bool::from_str(use_tmc_raw).unwrap_or(false);

		let vehice = match vehicle_raw {
			"car" => ::data::FLAG_CAR,
			"bike" => ::data::FLAG_BIKE,
			"walk" => ::data::FLAG_WALK,
			_ => ::data::FLAG_CAR
		};

		let metric = match (metric_raw, use_tmc) {
			("time", true) => edge_cost_tmc,
			("time", false) => edge_cost_time,
			("distance", _) => edge_cost_distance,
			_ => edge_cost_distance
		};

		println!("doing routing from {} to {} for vehicle {} with metric {} and tmc {}", source, target, vehicle_raw, metric_raw, use_tmc);

		let start = PreciseTime::now();
		let result = run_dijkstra(&data.routing_data, source, target, vehice, metric, tmc_state, use_tmc);
		let end = PreciseTime::now();
		//println!("route: {:?}", result);

		let result = RoutingResult { duration: start.to(end).num_milliseconds(), route: result };

		Ok(Response::with((status::Ok, json::encode(&result).unwrap())))
	} else {
		Ok(Response::with((status::InternalServerError)))
	}
}

fn run_dijkstra<F>(data: &::data::RoutingData, source_osm: i64, target_osm: i64, constraints: u8, cost_func: F, tmc_state: &RwLock<::data::TMCState>, use_tmc: bool) -> Option<Route>
	where F: Fn(&::data::RoutingEdge, &f64, &usize, &::data::TMCState) -> f64 {
	let vspeed = match constraints {
		::data::FLAG_CAR => 130.0 / 3.6,
		::data::FLAG_BIKE => 15.0 / 3.6,
		::data::FLAG_WALK => 5.0 / 3.6,
		_ => 130.0 / 3.6
	};

	let mut distance = vec![f64::INFINITY; data.internal_nodes.len()];
	let mut predecessor = vec![0; data.internal_nodes.len()];
	let mut predecessor_edge = vec![0; data.internal_nodes.len()];

	let source = data.osm_nodes.get(&source_osm).unwrap().internal_id;
	let target = data.osm_nodes.get(&target_osm).unwrap().internal_id;

	let tmc = tmc_state.read().unwrap();

	let mut heap = BinaryHeap::new();

	distance[source] = 0.0;
	heap.push(HeapEntry { node: source, cost: 0.0 });

	println!("begin dijkstra");

	while let Some(HeapEntry { node, cost }) = heap.pop() {
		if node == target {
			println!("found route");
			return build_route(source, target, &predecessor, &predecessor_edge, &data, &vspeed);
		}

		if cost > distance[node] { continue; }

		let (start, end) = offset_lookup(&node, &data);
		let edges = &data.internal_edges[start..end];

		for (i, edge) in edges.iter().enumerate() {
			if constraints & edge.constraints == 0 {
				continue;
			}

			let neighbor = HeapEntry { node: edge.target, cost: cost + cost_func(&edge, &vspeed, &(i + start), &tmc) };

			if neighbor.cost < distance[neighbor.node] {
				distance[edge.target] = neighbor.cost;
				predecessor[edge.target] = node;
				predecessor_edge[edge.target] = i + start;
				heap.push(neighbor);
			}
		}
	}
	println!("no route found");
	return None;
}


fn offset_lookup(node: &usize, data: &::data::RoutingData) -> (usize, usize) {
	let start = data.internal_offset[*node];
	let next_node = node + 1;
	let max_end = data.internal_offset[data.internal_offset.len() - 1];

	if next_node > data.internal_offset.len() - 1 {
		assert!(start <= max_end, "invalid offset lookup max!");

		return (start, max_end);
	}

	let end = data.internal_offset[next_node];

	assert!(start <= end, "invalid offset lookup!");

	return (start, end);
}


fn build_route(source: usize, target: usize, predecessor: &Vec<usize>, predecessor_edge: &Vec<usize>, data: &::data::RoutingData, vspeed: &f64) -> Option<Route> {
	let mut result = Route { distance: 0.0, time: 0.0, path: Vec::new() };

	let mut node = target;
	let mut edge = predecessor_edge[node];

	loop {
		if node == source {
			break;
		}

		let osm_id = data.internal_nodes[node];
		let ref pos = data.osm_nodes.get(&osm_id).unwrap().position;

		let mut speed = data.internal_edges[edge].speed;

		if *vspeed < speed {
			speed = *vspeed;
		}

		result.path.push([pos.lat, pos.lon]);
		result.distance += data.internal_edges[edge].length;
		result.time += data.internal_edges[edge].length / speed;

		node = predecessor[node];
		edge = predecessor_edge[node];
	}

	result.path.reverse();

	println!("build path, dist: {}, time: {}", result.distance, result.time);

	return Some(result);
}

fn edge_cost_distance(edge: &::data::RoutingEdge, _: &f64, _: &usize, _: &::data::TMCState) -> f64 {
	return edge.length;
}

fn edge_cost_tmc(edge: &::data::RoutingEdge, vspeed: &f64, edge_id: &usize, state: &::data::TMCState) -> f64 {
	let mut speed = edge.speed;

	let slowdown = match state.current_edge_events.get(edge_id) {
		Some(tmc_event) => {
			*tmc_event
		},
		None => 1.0,
	};

	if *vspeed < speed {
		speed = *vspeed;
	}

	return edge.length / f64::max(1.0, speed * slowdown);
}

fn edge_cost_time(edge: &::data::RoutingEdge, vspeed: &f64, _: &usize, _: &::data::TMCState) -> f64 {
	let mut speed = edge.speed;

	if *vspeed < speed {
		speed = *vspeed;
	}

	return edge.length / speed;
}

fn start_tmc_listener_thread(tmc_arc: Arc<RwLock<::data::TMCState>>, data_arc: Arc<::data::State>) {
	thread::spawn(move || {
		::tmc::run_tmc_thread(tmc_arc, data_arc);
	});
}

#[test]
fn test_dijkstra() {
	let data = ::parser::build_dummy_data();

	let path = run_dijkstra(&data, 5000, 5003, ::data::FLAG_CAR, edge_cost_time);

	println!("path: {:?}", path);
}
