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

TODO DOC

### Numbers

no slowdown entry for 929 of 1433 event codes (64.83%)
```
P1 | ways:         1457361
P1 | nodes_used:   8956154
P1 | edges:        0
P1 | nodes:        0
P1 | tmc_next_cnt: 426
P1 | tmc_tagged:   2666
P1 | duration:     PT34.562346491S
P2 | ways:       1457361
P2 | nodes_used: 0
P2 | edges:      0
P2 | nodes:      8956154
P2 | duration:   PT39.646970039S
P3 | ways:       0
P3 | nodes_used: 0
P3 | edges:      18945437
P3 | nodes:      8956154
P3 | duration:   PT43.043149923S
B  | edges:     18945437
B  | nodes:     8956154
B  | offset:    8956154
B  | osm_nodes: 8956154
B  | duration:  PT35.612597473S
G  | bounds:      BoundingBox { min_lat: 47.523482300000005, min_lon: 7.5025906, max_lat: 49.8142641, max_lon: 10.5342851 }
G  | bin_cnt_lat: 2186
G  | bin_cnt_lon: 1457
G  | duration:    PT1.114517948S
Writing state data to file state.bin.gz.. 
Writing state data to file state.bin.gz.. OK
```

4 TMC Tagging errors

https://wiki.openstreetmap.org/wiki/DE:TMC
Baden-Württemberg  (in OSM 2906 von 3226 TMC Objekten davon 38 Fehlerhaft  Eine Warnung)

https://wiki.openstreetmap.org/wiki/DE:Proposed_features/New_TMC_scheme