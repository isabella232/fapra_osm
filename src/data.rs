use std::collections::HashMap;
use std::cmp;
use std::f64;

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Position {
	pub lat: f64,
	pub lon: f64,
}

impl Position {
	pub fn distance(&self, other: &Position) -> f64 {
		let lat1 = self.lat;
		let lat2 = other.lat;
		let lng1 = self.lon;
		let lng2 = other.lon;

		let earth_radius: f64 = 6371000.0; //meters
		let d_lat = (lat2 - lat1).to_radians();
		let d_lng = (lng2 - lng1).to_radians();
		let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin() + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin() * (d_lng / 2.0).sin();
		let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
		let dist = earth_radius * c;

		dist
	}
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

pub struct TSMMapping {
	pub tsm_loc_to_node: HashMap<i64, HashSet<usize>>,
	// tsm_loc -> set<internal_node_id>
	pub tsm_loc_to_edge: HashMap<i64, HashSet<usize>>,
	// tsm_loc -> set<internal_edge_id>
}

pub struct TSMState {
	pub current_node_events: HashMap<usize, TSMEvent>,
	pub current_edge_events: HashMap<usize, TSMEvent>,
}

pub struct TSMEvent {
	pub desc: string,
	pub slowdown: f64
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

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct State {
	pub routing_data: RoutingData,
	pub grid: Grid,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Bin {
	pub nodes: Vec<i64>
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Grid {
	pub bbox: BoundingBox,
	pub bin_count_lat: usize,
	pub bin_count_lon: usize,
	pub bins: Vec<Bin>,
}

impl Grid {
	pub fn calc_bin_position(&self, lat_bin: usize, lon_bin: usize) -> usize {
		assert!(lat_bin < self.bin_count_lat && lon_bin < self.bin_count_lon, "invalid lat or lon bin given!");
		lat_bin * self.bin_count_lon + lon_bin
	}

	pub fn calc_bin_index(&self, position: &Position) -> (usize, usize) {
		let lat_bin = ((position.lat - self.bbox.min_lat) * self.bin_count_lat as f64) / (self.bbox.max_lat - self.bbox.min_lat);
		let lon_bin = ((position.lon - self.bbox.min_lon) * self.bin_count_lon as f64) / (self.bbox.max_lat - self.bbox.min_lon);

		(lat_bin as usize, lon_bin as usize)
	}

	pub fn find_closest_node(&self, position: &Position, routing_data: &RoutingData) -> i64 {
		let mut min_dist = f64::INFINITY;
		let mut min_node = -1;

		let (lat_bin, lon_bin) = self.calc_bin_index(position);

		let start_bin_lat = cmp::max(0, lat_bin - 1);
		let start_bin_lon = cmp::max(0, lon_bin - 1);

		let end_bin_lat = cmp::min(self.bin_count_lat, lat_bin + 2);
		let end_bin_lon = cmp::min(self.bin_count_lon, lon_bin + 2);

		for curr_bin_lat in start_bin_lat..end_bin_lat {
			for curr_bin_lon in start_bin_lon..end_bin_lon {
				let bin_index = self.calc_bin_position(curr_bin_lat, curr_bin_lon);

				for node in &self.bins[bin_index].nodes {
					if let Some(candidate) = routing_data.osm_nodes.get(node) {
						let candidate_distance = position.distance(&candidate.position);
						if candidate_distance < min_dist {
							min_dist = candidate_distance;
							min_node = *node;
						}
					}
				}
			}
		}
		min_node
	}
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct BoundingBox {
	pub min_lat: f64,
	pub min_lon: f64,
	pub max_lat: f64,
	pub max_lon: f64
}