use futures::TryStreamExt as _;
use hyper::{Body, Request, Response, Server, Method, StatusCode };
use hyper::service::{make_service_fn, service_fn};

async fn echo(req: Request<Body>)  -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => 
            Ok(Response::new(Body::from("try POST-ing data from /echo"))) ,
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())) ,
        (&Method::POST, "/echo/uppercase") => {
            let chunk_stream = req.into_body().map_ok(|chunk| {
                    chunk.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Vec<u8>>()
                });
            Ok(Response::new(Body::wrap_stream(chunk_stream)))
        },
        (&Method::POST, "/echo/reverse") => {
            let full_body = hyper::body::to_bytes(req.into_body()).await?;
            let reversed = full_body.iter()
                .rev()
                .cloned()
                .collect::<Vec<u8>>();
            Ok(Response::new(Body::from(reversed)))
        },
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let addr = (([127, 0, 0, 1], 3000)).into();
    
    let service = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(echo))
    });

    let server = Server::bind(&addr).serve(service);

    println!("listening to server on port {}", addr);
    server.await?;
    Ok(())
}
