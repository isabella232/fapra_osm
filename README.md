# fapra_osm

Routing based on OpenStreetMap data extracted from .pbf files, written in Rust.

## build & run

* Download and install Rust (>1.9.0): https://www.rust-lang.org/downloads.html
* Download region pbf file from http://download.geofabrik.de/
* Clone repo: `git clone git@github.com:s1mpl3x/fapra_osm.git`
* run `cargo build --release`
* run `cargo run --release /path/to/your/xxx-latest.osm.pbf`

The last command will start the application. If no `state.bin.gz` file is present in the project root, the given pbf file will be parsed and the resulting state data will be saved to `state.bin.gz`.
Once the state has been loaded or parsed and the line `server running on http://localhost:8080/` was printed, the ui can be accessed at http://localhost:8080/

## screenshot

![screenshot](https://i.imgur.com/ZuoCnk1.png)

## TMC

This application can also consider TMC events for route calculation. See the report for more details.
