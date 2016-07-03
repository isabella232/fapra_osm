use std::collections::HashMap;

#[derive(Debug, Clone, Copy, RustcEncodable, RustcDecodable)]
pub struct Position {
	pub lat: f64,
	pub lon: f64,
}

pub const FLAG_CAR: u8 = 0b00000001;
pub const FLAG_BIKE: u8 = 0b00000010;
pub const FLAG_WALK: u8 = 0b00000100;


#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct RoutingEdge {
	pub target: usize,
	pub length: f64,
	pub speed: f64,
	pub constraints: u8,
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct OsmNode {
	pub position: Position,
	pub internal_id: usize
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct RoutingData {
	// relevant nodes and their position
	pub osm_nodes: HashMap<i64, OsmNode>,
	//[n_id] -> osm_n_id
	pub internal_nodes: Vec<i64>,
	// [e_id] -> target_n_id|length|constraints
	pub internal_edges: Vec<RoutingEdge>,
	// [n_id] -> e_id
	pub internal_offset: Vec<usize>,
}