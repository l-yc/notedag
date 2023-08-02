extern crate pretty_env_logger;
#[macro_use] extern crate log;

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

    let port = env::var_os("PORT")
        .map(|s| s.into_string().unwrap().parse().unwrap())
        .unwrap_or(8080);

    let app_root = env::var_os("APP")
        .map(|s| s.into_string().unwrap())
        .unwrap_or("../notedag-frontend/build".into());

    let (notify_shutdown_tx, notify_shutdown_rx) = tokio::sync::broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = tokio::sync::mpsc::channel(1);

    let api = filters::api(notify_shutdown_rx, shutdown_complete_tx)
        .with(warp::cors().allow_any_origin());
    let app = warp::fs::dir(app_root.clone());

    let routes = app.or(api)
        .with(warp::log("notedag"));

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
    info!("serving from {}", app_root); 
    server.await;
}

mod filters;
mod models;
mod handlers;
mod kernel;
