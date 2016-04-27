extern crate osmpbfreader;

use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;

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


    let firstParseResult = first_parse(&pbf_file);

    println!("ways:  {}", firstParseResult.filtered_way_ids.len());
    println!("nodes: {}", firstParseResult.node_ref_count.len());
}

fn first_parse(pbf_file: &File) -> FirstParseResult {
    let mut result = FirstParseResult { node_ref_count: HashMap::new(), filtered_way_ids: Vec::new() };

    let mut pbf = OsmPbfReader::new(pbf_file);

    for obj in pbf.iter() {
        match obj {
            OsmObj::Way(way) => {
                if filter_way(&way) {
                    result.filtered_way_ids.push(way.id);
                    for node in way.nodes {
                        let nodeEntry = result.node_ref_count.entry(node).or_insert(0);
                        *nodeEntry += 1;
                    }
                }
            }
            _ => {}
        }
    }

    return result;
}

fn filter_way(way: &osmpbfreader::Way) -> bool {
    if let Some(value) = way.tags.get("highway") {
        let invalid_values = vec!["services", "pedestrian", "raceway", "footway", "path", "steps", "bridleway", "construction"];

        return !invalid_values.contains(&value.as_str());
    } else {
        return false;
    }
}