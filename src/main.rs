extern crate osmpbfreader;
extern crate time;

use time::PreciseTime;
use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use std::collections::HashSet;

use osmpbfreader::OsmObj;
use osmpbfreader::OsmPbfReader;

#[derive(Debug, Clone, Copy)]
struct Position {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone, Copy)]
struct RoutingEdge {
    id_from: i64,
    id_to: i64,
    length: f64
}

struct FirstParseResult {
    // used node-ids
    nodes_used: HashSet<i64>,
    // "useful" ways
    filtered_ways: HashSet<i64>
}

struct SecondParseResult {
    // relevant nodes and their position
    nodes: HashMap<i64, Position>,
}

struct ThirdParseResult {
    // edges
    edges: Vec<RoutingEdge>
}

struct RoutingData {
    // relevant nodes and their position
    osm_nodes: HashMap<i64, Position>,
    //[n_id] -> osm_n_id
    internal_nodes: Vec<i64>,
    // [e_id] -> osm_n_id target
    internal_edges: Vec<i64>,
    // [n_id] -> e_id
    internal_offset: Vec<i64>,
    // e_id -> length
    internal_length: Vec<f64>
}


fn main() {
    let default_file = OsString::from("/home/zsdn/germany-latest.osm.pbf".to_string());
    let args: Vec<OsString> = std::env::args_os().collect();
    match args.len() {
        1 => {
            read_file(&default_file);
        }
        2 => {
            read_file(&args[1]);
        }
        _ => {},
    };
}

fn read_file(filename: &OsString) {
    let start_p1 = PreciseTime::now();
    let firstParseResult = first_parse(&filename);
    let end_p1 = PreciseTime::now();

    println!("P1 | ways:  {}", firstParseResult.filtered_ways.len());
    println!("P1 | nodes: {}", firstParseResult.nodes_used.len());
    println!("P1 | duration: {}", start_p1.to(end_p1));

    let start_p2 = PreciseTime::now();
    let secondParseResult = second_parse(&filename, &firstParseResult);
    let end_p2 = PreciseTime::now();

    println!("P2 | nodes: {}", secondParseResult.nodes.len());
    println!("P2 | duration: {}", start_p2.to(end_p2));

    let start_p3 = PreciseTime::now();
    let thirdParseResult = third_parse(&filename, &firstParseResult, &secondParseResult);
    let end_p3 = PreciseTime::now();

    println!("P3 | edges:  {}", thirdParseResult.edges.len());
    println!("P3 | duration: {}", start_p3.to(end_p3));
}


fn first_parse(filename: &OsString) -> FirstParseResult {
    let pbf_file = File::open(&Path::new(filename)).unwrap();
    let mut result = FirstParseResult { nodes_used: HashSet::new(), filtered_ways: HashSet::new() };

    let mut invalid_values = HashSet::new();
    init_filter_list(&mut invalid_values);

    let mut pbf = OsmPbfReader::new(pbf_file);

    for obj in pbf.iter() {
        match obj {
            OsmObj::Way(way) => {
                if filter_way(&way, &invalid_values) {
                    for node in way.nodes {
                        result.nodes_used.insert(node);
                    }
                    result.filtered_ways.insert(way.id);
                }
            }
            _ => {}
        }
    }

    return result;
}

fn second_parse(filename: &OsString, input: &FirstParseResult) -> SecondParseResult {
    let pbf_file = File::open(&Path::new(filename)).unwrap();
    let mut pbf = OsmPbfReader::new(pbf_file);
    let mut result = SecondParseResult { nodes: HashMap::new() };

    for obj in pbf.iter() {
        match obj {
            OsmObj::Node(node) => {
                if input.nodes_used.contains(&node.id) {
                    result.nodes.insert(node.id, Position { lat: node.lat, lon: node.lon });
                }
            }
            _ => {}
        }
    }

    return result;
}

fn third_parse(filename: &OsString, first: &FirstParseResult, second: &SecondParseResult) -> ThirdParseResult {
    let pbf_file = File::open(&Path::new(filename)).unwrap();
    let mut pbf = OsmPbfReader::new(pbf_file);
    let mut result = ThirdParseResult { edges: Vec::new() };

    for obj in pbf.iter() {
        match obj {
            OsmObj::Way(way) => {
                if first.filtered_ways.contains(&way.id) {
                    for node_pair in way.nodes.windows(2) {
                        if let (Some(from), Some(to)) = (node_pair.first(), node_pair.last()) {
                            if let (Some(from_node), Some(to_node)) = (second.nodes.get(from), second.nodes.get(to)) {
                                let edge_length = calculate_distance(from_node, to_node);
                                let edge = RoutingEdge { id_from: from.clone(), id_to: to.clone(), length: edge_length.clone() };
                                result.edges.push(edge);

                                if let Some(val) = way.tags.get("oneway") {
                                    if val == "yes" {
                                        let edge_reverse = RoutingEdge { id_from: to.clone(), id_to: from.clone(), length: edge_length.clone() };
                                        result.edges.push(edge_reverse);
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

    return result;
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

fn filter_way(way: &osmpbfreader::Way, invalid_values: &HashSet<&str>) -> bool {
    if let Some(value) = way.tags.get("highway") {
        return !invalid_values.contains(&value.as_str());
    } else {
        return false;
    }
}