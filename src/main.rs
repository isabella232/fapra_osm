extern crate osmpbfreader;

use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use std::collections::HashSet;

use osmpbfreader::OsmObj;
use osmpbfreader::OsmPbfReader;

/*
 * 1. parse ways; throw away those that are not roads, and for the others, remember the node IDs they consist of, by incrementing a "link counter" for each node referenced.
 * 2. parse all ways a second time; a way will normally become one edge, but if any nodes apart from the first and the last have a link counter greater than one, then split the way into two edges at that point. Nodes with a link counter of one and which are neither first nor last can be thrown away unless you need to compute the length of the edge.
 * 3. (if you need geometry for your graph nodes) parse the nodes section of the XML now, recording coordinates for all nodes that you have retained.
 */

struct RoutingNode {
    id: i64,
    lat: f64,
    lon: f64,
}

struct RoutingEdge {
    id_from: i64,
    id_to: i64,
    length: f64
}

struct FirstParseResult {
    node_ref_count: HashMap<i64, i32>,
    filtered_way_ids: Vec<i64>
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
    let pbf_file = File::open(&Path::new(filename)).unwrap();

    //let firstParseResult = first_parse(&pbf_file);

    //println!("ways:  {}", firstParseResult.filtered_way_ids.len());
    //println!("nodes: {}", firstParseResult.node_ref_count.len());
    let n1 = osmpbfreader::Node { id: 1, lat: 48.94647, lon: 9.09309, tags: osmpbfreader::Tags::new() };
    let n2 = osmpbfreader::Node { id: 1, lat: 48.74537, lon: 9.10711, tags: osmpbfreader::Tags::new() };

    println!("distance: {}", calculate_distance(&n1, &n2));
}

fn calculate_distance(node1: &osmpbfreader::Node, node2: &osmpbfreader::Node) -> f64 {
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


fn first_parse(pbf_file: &File) -> FirstParseResult {
    let mut result = FirstParseResult { node_ref_count: HashMap::new(), filtered_way_ids: Vec::new() };


    let mut invalid_values = HashSet::new();
    init_filter_list(&mut invalid_values);

    let mut pbf = OsmPbfReader::new(pbf_file);

    for obj in pbf.iter() {
        match obj {
            OsmObj::Way(way) => {
                if filter_way(&way, &invalid_values) {
                    result.filtered_way_ids.push(way.id);
                    for node in way.nodes {
                        let node_entry = result.node_ref_count.entry(node).or_insert(0);
                        *node_entry += 1;
                    }
                }
            }
            _ => {}
        }
    }

    return result;
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