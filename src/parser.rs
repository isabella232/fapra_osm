use time::PreciseTime;
use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use std::collections::HashSet;

use osmpbfreader::OsmObj;
use osmpbfreader::OsmPbfReader;

#[derive(Debug, Clone, Copy, RustcEncodable, RustcDecodable)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Copy)]
struct ParsedEdge {
    id_from: i64,
    id_to: i64,
    length: f64,
    constraints: u8,
}

struct ParseData {
    // used node-ids
    nodes_used: HashSet<i64>,
    // "useful" ways
    filtered_ways: HashMap<i64, u8>,
    // relevant nodes and their position
    nodes: HashMap<i64, Position>,
    // edges
    edges: Vec<ParsedEdge>
}


pub const FLAG_CAR: u8 = 0b00000001;
pub const FLAG_BIKE: u8 = 0b00000010;
pub const FLAG_WALK: u8 = 0b00000100;


#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct RoutingEdge {
    pub target: usize,
    pub length: f64,
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

pub fn read_file(filename: &OsString) -> RoutingData {
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

    return routing_data;
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

pub fn build_dummy_data() -> RoutingData {
    let edge_vec = vec![ParsedEdge{id_from: 5000, id_to: 5001, length: 1.0, constraints: FLAG_CAR },
                        ParsedEdge{id_from: 5000, id_to: 5002, length: 1.0, constraints: FLAG_CAR },
                        ParsedEdge{id_from: 5002, id_to: 5001, length: 1.0, constraints: FLAG_CAR },
                        ParsedEdge{id_from: 5002, id_to: 5003, length: 1.0, constraints: FLAG_CAR },
                        ParsedEdge{id_from: 5003, id_to: 5000, length: 1.0, constraints: FLAG_CAR },
                        ParsedEdge{id_from: 5003, id_to: 5004, length: 1.0, constraints: FLAG_CAR },
    ];


    let mut nodes_map = HashMap::new();

    nodes_map.insert(5000, Position { lat: 0.0, lon: 0.0 });
    nodes_map.insert(5001, Position { lat: 0.0, lon: 0.0 });
    nodes_map.insert(5002, Position { lat: 0.0, lon: 0.0 });
    nodes_map.insert(5003, Position { lat: 0.0, lon: 0.0 });
    nodes_map.insert(5004, Position { lat: 0.0, lon: 0.0 });

    let parse_result = ParseData { nodes: nodes_map, edges: edge_vec, filtered_ways: HashMap::new(), nodes_used: HashSet::new() };

    build_routing_data(parse_result)
}


fn first_parse(filename: &OsString, parse_result: &mut ParseData) {
    let pbf_file = File::open(&Path::new(filename)).unwrap();

    let mut invalid_values = HashSet::new();
    init_filter_list(&mut invalid_values);

    let mut pbf = OsmPbfReader::new(pbf_file);

    for obj in pbf.iter() {
        match obj {
            OsmObj::Way(way) => {
                if let Some(constraints) = filter_way(&way, &invalid_values) {
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
                    parse_result.nodes.insert(node.id, Position { lat: node.lat, lon: node.lon });
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
                if let Some(constraints) = parse_result.filtered_ways.remove(&way.id) {
                    for node_pair in way.nodes.windows(2) {
                        if let (Some(from), Some(to)) = (node_pair.first(), node_pair.last()) {
                            if let (Some(from_node), Some(to_node)) = (parse_result.nodes.get(from), parse_result.nodes.get(to)) {
                                let edge_length = calculate_distance(from_node, to_node);
                                let edge = ParsedEdge { id_from: from.clone(), id_to: to.clone(), length: edge_length.clone(), constraints: constraints };
                                parse_result.edges.push(edge);

                                if let Some(val) = way.tags.get("oneway") {
                                    if val == "yes" {
                                        let edge_reverse = ParsedEdge { id_from: to.clone(), id_to: from.clone(), length: edge_length.clone(), constraints: constraints };
                                        parse_result.edges.push(edge_reverse);
                                    }
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

fn build_routing_data(mut parse_result: ParseData) -> RoutingData {
    let mut routing_data = RoutingData { osm_nodes: HashMap::new(), internal_nodes: Vec::new(), internal_edges: Vec::new(), internal_offset: vec![usize::max_value(); parse_result.nodes.len()] };

    parse_result.edges.sort_by(|a, b| b.id_from.cmp(&a.id_from));

    for node in parse_result.nodes.keys() {
        routing_data.internal_nodes.push(node.clone());
    }

    routing_data.internal_nodes.sort();


    for (i, node) in routing_data.internal_nodes.iter().enumerate() {
        if let Some(pos) = parse_result.nodes.remove(node) {
            routing_data.osm_nodes.insert(node.clone(), OsmNode { position: pos, internal_id: i });
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

                routing_data.internal_edges.push(RoutingEdge { target: internal_target, length: edge.length, constraints: edge.constraints });
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


fn calculate_distance(node1: &Position, node2: &Position) -> f64 {
    let lat1 = node1.lat;
    let lat2 = node2.lat;
    let lng1 = node1.lon;
    let lng2 = node2.lon;

    let earth_radius: f64 = 6371000.0; //meters
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin() + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin() * (d_lng / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    let dist = earth_radius * c;

    return dist;
}

fn init_filter_list(invalid_values: &mut HashSet<&str>) {
    invalid_values.insert("services");
    invalid_values.insert("pedestrian");
    invalid_values.insert("raceway");
    invalid_values.insert("footway");
    invalid_values.insert("path");
    invalid_values.insert("steps");
    invalid_values.insert("bridleway");
    invalid_values.insert("construction");
}

fn filter_way(way: &::osmpbfreader::Way, invalid_values: &HashSet<&str>) -> Option<u8> {
    if let Some(value) = way.tags.get("highway") {
        if !invalid_values.contains(&value.as_str()) {
            // TODO really check constraints
            return Some(FLAG_CAR | FLAG_WALK | FLAG_BIKE);
        }
    }
    return None;
}
