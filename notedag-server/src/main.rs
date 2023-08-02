extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::env;
use warp::Filter;

mod app {
    use rust_embed::RustEmbed;
    use warp::{http::header::HeaderValue, path::Tail, reply::Response, Filter, Rejection, Reply};

    #[derive(RustEmbed)]
    #[folder = "../notedag-frontend/build/"]
    struct Asset;

    async fn serve_index() -> Result<impl Reply, Rejection> {
        serve_impl("index.html")
    }

    async fn serve(path: Tail) -> Result<impl Reply, Rejection> {
        serve_impl(path.as_str())
    }

    fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
        info!("serving path {}", path);
        let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
        let mime = mime_guess::from_path(path).first_or_octet_stream();

        let mut res = Response::new(asset.data.into());
        res.headers_mut().insert("Content-Type", HeaderValue::from_str(mime.as_ref()).unwrap());
        Ok(res)
    }

    pub fn main() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let index_html = warp::path::end().and_then(serve_index);
        let dist = warp::path::tail().and_then(serve);

        index_html.or(dist)
    }
}


#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "notedag=info");
    }
    pretty_env_logger::init();

    let port = env::var_os("PORT")
        .map(|s| s.into_string().unwrap().parse().unwrap())
        .unwrap_or(8080);

    let (notify_shutdown_tx, notify_shutdown_rx) = tokio::sync::broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = tokio::sync::mpsc::channel(1);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["GET", "POST"]);
    let api = filters::api(notify_shutdown_rx, shutdown_complete_tx)
        .with(cors);
    let app = app::main();

    let routes = app.or(api)
        .with(warp::log("notedag_api"));

    let (_addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], port), async move {
            loop {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen to shutdown signal");
                info!("ctrl c received, shutting down gracefully");
                let _ = notify_shutdown_tx.send(());
                let _ = shutdown_complete_rx.recv().await;
                break;
            }
        });

    info!("listening on {}", _addr); 
    server.await;
}

mod filters;
mod models;
mod handlers;
mod kernel;
