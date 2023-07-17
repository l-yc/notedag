use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "notedag=info");
    }
    pretty_env_logger::init();

    let api = filters::api();

    let routes = api.with(warp::log("todos"));

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

mod filters;
mod models;
mod handlers;
