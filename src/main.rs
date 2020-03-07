use futures::TryStreamExt as _;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use slab::Slab;
use std::fmt;
use std::sync::{Arc, Mutex};

type UserId = u64;

#[derive(Debug)]
struct UserData;

impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{}")
    }
}
const USER_PATH: &str = "/user/";
type UserDb = Arc<Mutex<Slab<UserData>>>;

const INDEX: &'static str = "
    <!doctype HTML>
    <html>
        <head>
            <title>Rust Microservice</title>
        </head>
        <body>
            <h3>Rust Microservice</h3>
        </body>
    </html>
";

async fn request_handler(req: Request<Body>, user_db: UserDb) -> Result<Response<Body>, hyper::Error> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Response::new(Body::from(
            "try POST-int data from /echo, e.g. curl -XPOST -d 'hello world' localhost:3000/echo",
        )),
        (method, path) if path.starts_with(USER_PATH) => {
            let user_id = path
                .trim_start_matches(USER_PATH)
                .parse::<UserId>()
                .ok()
                .map(|x| x as usize);

            let mut users = user_db.lock().unwrap();
            match (method, user_id) {
                (&Method::POST, None) => {
                    let id = users.insert(UserData {});
                    Response::new(id.to_string().into())
                }
                (&Method::POST, Some(_)) => response_with_code(StatusCode::BAD_REQUEST),
                (&Method::GET, Some(id)) => {
                    if let Some(data) = users.get(id) {
                        Response::new(data.to_string().into())
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                (&Method::PUT, Some(id)) => {
                    if let Some(user) = users.get_mut(id) {
                        *user = UserData;
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                (&Method::DELETE, Some(id)) => {
                    if users.contains(id) {
                        users.remove(id);
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                _ => response_with_code(StatusCode::METHOD_NOT_ALLOWED),
            }
        }
        (&Method::POST, "/echo") => Response::new(Body::from(req.into_body())),
        (&Method::POST, "/echo/uppercase") => {
            let chunk_stream = req.into_body().map_ok(|chunk| {
                chunk
                    .iter()
                    .map(|byte| byte.to_ascii_uppercase())
                    .collect::<Vec<u8>>()
            });

            Response::new(Body::wrap_stream(chunk_stream))
        }
        (&Method::POST, "/echo/reverse") => {
            let full_body = hyper::body::to_bytes(req.into_body()).await?;
            let reversed = full_body.iter().rev().cloned().collect::<Vec<u8>>();
            Response::new(Body::from(reversed))
        }
        (&Method::GET, "/home") => Response::new(Body::from(INDEX)),
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            not_found
        }
    };
    Ok(response)
}

fn response_with_code(status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let user_db = Arc::new(Mutex::new(Slab::new()));
    let service = make_service_fn(move|_conn| {
        let user_db = Arc::clone(&user_db);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let users = user_db.clone();
                request_handler(req, users)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);
    println!("listening to server on {}", addr);
    server.await?;
    Ok(())
}
