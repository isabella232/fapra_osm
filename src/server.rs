use std::path::Path;
use std::sync::Arc;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::f64;
use iron::prelude::*;
use iron::status;
use staticfile::Static;
use mount::Mount;
use ordered_float::OrderedFloat;
use urlencoded::UrlEncodedQuery;
use rustc_serialize::json;

#[derive(Copy, Clone)]
struct HeapEntry {
	node: usize,
	cost: f64,
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

impl Eq for HeapEntry {
}

impl PartialEq for HeapEntry {
	fn eq(&self, other: &HeapEntry) -> bool {
		return (self.node == other.node) & &(OrderedFloat(other.cost).eq(&OrderedFloat(self.cost)))
	}
}


pub fn start(data: ::data::RoutingData) {
	let data_wrapped = Arc::new(data);
	let data_wrapped_2 = data_wrapped.clone();
	let data_wrapped_3 = data_wrapped.clone();

	let mut mount = Mount::new();

	mount.mount("/", Static::new(Path::new("web/")));
	mount.mount("/api/hello", move |r: &mut Request| get_hello(r, &data_wrapped));
	mount.mount("/api/graph", move |r: &mut Request| get_graph(r, &data_wrapped_2));
	mount.mount("/api/route", move |r: &mut Request| get_route(r, &data_wrapped_3));

	println!("server running on http://localhost:8080/");

	Iron::new(mount).http("127.0.0.1:8080").unwrap();
}

fn get_hello(req: &mut Request, data: &::data::RoutingData) -> IronResult<Response> {
	println!("Running get_hello handler, URL path: {:?}", req.url.path);
	Ok(Response::with((status::Ok, format!("HI! nodes: {}, edges: {}", data.internal_nodes.len(), data.internal_edges.len()))))
}

fn get_graph(req: &mut Request, data: &::data::RoutingData) -> IronResult<Response> {
	println!("Running get_graph handler, URL path: {:?}", req.url.path);
	Ok(Response::with((status::Ok, format!("nodes: {}, edges: {}", data.internal_nodes.len(), data.internal_edges.len()))))
}

fn get_route(req: &mut Request, data: &::data::RoutingData) -> IronResult<Response> {
	if let Ok(ref query_map) = req.get_ref::<UrlEncodedQuery> () {
		let source = itry!(query_map.get("source").unwrap().first().unwrap().parse::< i64 >());
		let target = itry!(query_map.get("target").unwrap().first().unwrap().parse::< i64 > ());


		if let Some(path) = run_dijkstra(&data, source, target, ::data::FLAG_CAR, edge_cost_time) {
			let mut pos_vec = Vec::new();
			for pos in &path {
				pos_vec.push(vec![pos.lat, pos.lon]);
			}

			Ok(Response::with((status::Ok, json::encode(&pos_vec).unwrap())))
		} else {
			Ok(Response::with((status::NotFound)))
		}
	} else {
		Ok(Response::with((status::InternalServerError)))
	}
}

fn run_dijkstra<F>(data: &::data::RoutingData, source_osm: i64, target_osm: i64, constraints: u8, cost_func: F) -> Option<Vec<::data::Position>>
	where F: Fn(&::data::RoutingEdge) -> f64 {
	let mut distance = vec![f64::INFINITY; data.internal_nodes.len()];
	let mut predecessor = vec![0; data.internal_nodes.len()];

	let source = data.osm_nodes.get(&source_osm).unwrap().internal_id;
	let target = data.osm_nodes.get(&target_osm).unwrap().internal_id;

	let mut heap = BinaryHeap::new();

	distance[source] = 0.0;
	heap.push(HeapEntry { node: source, cost: 0.0 });

	while let Some(HeapEntry { node, cost }) = heap.pop() {
		if node == target {
			return build_way(source, target, &predecessor, &data);
		}

		if cost > distance[node] { continue; }

		let (start, end) = offset_lookup(&node, &data);


		let edges = &data.internal_edges[start..end];


		for edge in edges {
			if constraints & edge.constraints == 0 {
				continue;
			}
			let neighbor = HeapEntry { node: edge.target, cost: cost + cost_func(&edge) };

			if neighbor.cost < distance[neighbor.node] {
				heap.push(neighbor);
				distance[edge.target] = neighbor.cost;
				predecessor[edge.target] = node;
			}
		}
	}

	return None;
}

fn offset_lookup(node: &usize, data: &::data::RoutingData) -> (usize, usize) {
	let start = data.internal_offset[*node];
	let next_node = node + 1;
	let max_end = data.internal_offset.len();

	if next_node >= max_end {
		return (start, max_end);
	}

	let end = data.internal_offset[next_node];

	return (start, end);
}


fn build_way(source: usize, target: usize, predecessor: &Vec<usize>, data: &::data::RoutingData) -> Option<Vec<::data::Position>> {
	let mut result = Vec::new();

	let mut node = target;

	loop {
		let osm_id = data.internal_nodes[node];

		let pos = data.osm_nodes.get(&osm_id).unwrap().position;

		result.push(pos);

		if node == source {
			break;
		}

		node = predecessor[node];
	}


	result.reverse();

	return Some(result);
}

fn edge_cost_distance(edge: &::data::RoutingEdge) -> f64 {
	return edge.length;
}

fn edge_cost_time(edge: &::data::RoutingEdge) -> f64 {
	return edge.length / edge.speed;
}

#[test]
fn test_dijkstra() {
	let data = ::parser::build_dummy_data();

	let path = run_dijkstra(&data, 5000, 5003, ::parser::FLAG_CAR);

	println!("path: {:?}", path);
}
