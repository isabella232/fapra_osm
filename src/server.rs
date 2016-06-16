use std::path::Path;

use iron::{Iron, Request, Response, IronResult};
use iron::status;
use staticfile::Static;
use mount::Mount;

pub fn start() {
    let mut mount = Mount::new();

    mount.mount("/", Static::new(Path::new("web/")));
    mount.mount("/api", send_hello);

    println!("server running on http://localhost:8080/");

    Iron::new(mount).http("127.0.0.1:8080").unwrap();
}

fn send_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {:?}", req.url.path);
    Ok(Response::with((status::Ok, "Hello!")))
}
