use parking_lot::RwLock;
use std::sync::Arc;
use std::fmt;

use warp;
use warp::{http::StatusCode, Filter, Buf, http::Response, Reply, Rejection, reject};
use std::convert::Infallible;

pub trait UserAuthStore: fmt::Debug + Send + Sync {

    fn get_authtoken(&self) -> Option<String>;
}

type Store = Arc<RwLock<Box<dyn UserAuthStore>>>;


#[derive(Debug)]
struct HayStackAuthToken;

impl reject::Reject for HayStackAuthToken {}


async fn hello(store: Store) -> Result<impl warp::Reply, warp::Rejection> {

    let response = warp::reply::with_status("Hello", http::StatusCode::from_u16(200).unwrap());

    println!("response: {:?} store: {:?}", response, store);

    return Ok(response);
}

pub fn haystack_auth_header(store: Store) -> impl Filter<Extract = (Store,), Error = Rejection> + Clone {

   let tmp = store.clone();

    warp::header("Authorization").and_then (|auth_header: String| async move {

            if tmp.read().get_authtoken().is_none() {
                return Err(reject::custom(HayStackAuthToken));   
            }
            
            Ok(tmp.clone())
   
        }
    )
}

fn with_store(store: Store) -> impl Filter<Extract = (Store,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}


#[derive(Clone, Debug)]
struct DemoAuthDetails {
}

impl DemoAuthDetails {
    fn new() -> Self {
        DemoAuthDetails {}
    }
}

impl UserAuthStore for DemoAuthDetails {
  
    fn get_authtoken(&self) -> Option<String> {
        None
    }
}


#[tokio::main]
async fn main() {

    let store = Arc::new(RwLock::new(Box::new(DemoAuthDetails::new()) as Box<dyn UserAuthStore>));

    let hello_route = warp::path("hello")
            .and(warp::path::end())
            .and(haystack_auth_header(store.clone()))
            .and_then(hello);

    let api = hello_route;

    // View access logs by setting `RUST_LOG=todos`.
    let routes = api.with(warp::log("webserver")).with(warp::cors().allow_any_origin());

    println!("Calling warp::serve");

    warp::serve(routes).run(([0, 0, 0, 0], 4337)).await;


}


