extern crate osmpbfreader;

fn count<F: Fn(&osmpbfreader::Tags) -> bool>(filter: F, filename: &std::ffi::OsStr) {
    let r = std::fs::File::open(&std::path::Path::new(filename)).unwrap();
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);
    let mut nb_nodes = 0;
    let mut sum_lon = 0.;
    let mut sum_lat = 0.;
    let mut nb_ways = 0;
    let mut nb_way_nodes = 0;
    let mut nb_rels = 0;
    let mut nb_rel_refs = 0;

    let mut cnt_total = 0;

    for obj in pbf.iter() {
        if !filter(obj.tags()) { continue; }
        //info!("{:?}", obj);
        match obj {
            osmpbfreader::OsmObj::Node(node) => {
                nb_nodes += 1;
                sum_lon += node.lon;
                sum_lat += node.lat;
            }
            osmpbfreader::OsmObj::Way(way) => {
                nb_ways += 1;
                nb_way_nodes += way.nodes.len();
            }
            osmpbfreader::OsmObj::Relation(rel) => {
                nb_rels += 1;
                nb_rel_refs += rel.refs.len();
            }
        }
        cnt_total += 1;

        if cnt_total % 100000 == 0 {
            println!("cnt: {}", cnt_total);
        }
    }
    println!("{} nodes, mean coord: {}, {}.",
    nb_nodes, sum_lat / nb_nodes as f64, sum_lon / nb_nodes as f64);
    println!("{} ways, mean |nodes|: {}",
    nb_ways, nb_way_nodes as f64 / nb_ways as f64);
    println!("{} relations, mean |references|: {}",
    nb_rels, nb_rel_refs as f64 / nb_rels as f64);
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    match args.len() {
        2 => {
            println!("counting objects...");
            count(|_| true, &args[1]);
        }
        3 => {
            let key = args[2].to_str().unwrap();
            println!("counting objects with \"{}\" in tags...", key);
            count(|tags| tags.contains_key(key), &args[1]);
        }
        4 => {
            let key = args[2].to_str().unwrap();
            let val = args[3].to_str().unwrap();
            println!("counting objects with tags[\"{}\"] = \"{}\"...", key, val);
            count(|tags| tags.get(key).map(|v| *v == val).unwrap_or(false), &args[1]);
        }
        _ => println!("usage: count filename [key [value]]", ),
    };
}