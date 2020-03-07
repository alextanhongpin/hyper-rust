use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn hello_world(_req: Request<Body>)  -> Result<Response<Body>, Infallible> {
    Ok(Response::new("hello world".into()))
}


#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("listening to server on port *:{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
