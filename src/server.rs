use std::path::Path;
use std::sync::Arc;
use iron::{Iron, Request, Response, IronResult};
use iron::status;
use staticfile::Static;
use mount::Mount;

#[derive(Debug)]
struct Server {
    graph: ::parser::RoutingData,
}

pub fn start(data: ::parser::RoutingData) {
    let server = Arc::new(Server { graph: data });
    let server_2 = server.clone();

    let mut mount = Mount::new();

    mount.mount("/", Static::new(Path::new("web/")));
    mount.mount("/api/hello", move |r: &mut Request| get_hello(r, &server));
    mount.mount("/api/graph", move |r: &mut Request| get_graph(r, &server_2));

    println!("server running on http://localhost:8080/");

    Iron::new(mount).http("127.0.0.1:8080").unwrap();
}

fn get_hello(req: &mut Request, server: &Server) -> IronResult<Response> {
    println!("Running get_hello handler, URL path: {:?}", req.url.path);
    Ok(Response::with((status::Ok, "Hello!")))
}

fn get_graph(req: &mut Request, server: &Server) -> IronResult<Response> {
    println!("Running get_graph handler, URL path: {:?}", req.url.path);
    Ok(Response::with((status::Ok, format!("{:?}", server))))
}