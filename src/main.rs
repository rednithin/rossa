use async_std::{net::SocketAddr, sync::Arc};
use clap::{self, Clap};
use dotenv;
use log;
use pretty_env_logger;
use tera::{Context, Tera};
use warp::Filter;

/// A SimpleHTTPServer clone written in Rust.
/// This is also inspired by gossa - https://github.com/pldubouilh/gossa.
#[derive(Clap)]
#[clap(
    version = "1.0.0",
    author = "P G Nithin Reddy <reddy.nithinpg@gmail.com>"
)]
struct Opts {
    /// `address` must be of the form <IP>:<Port>
    #[clap(short, long, default_value = "127.0.0.1:3030")]
    address: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap_or_default();
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();
    let bind_address: SocketAddr = opts.address.parse().expect("Invalid Bind Address");

    // Loading Tera templates once
    let tera = Arc::new(match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    });

    // Different types of Routes
    let static_files_route = warp::path("static").and(warp::fs::dir("."));

    let invalid_static_files_route =
        warp::path("static").map(|| format!("Invalid static file path"));

    let dynamic_route = warp::path::full()
        .and(warp::any().map(move || Arc::clone(&tera)))
        .map(|path, tera: Arc<Tera>| {
            log::info!("The path is {:?}", path);
            warp::reply::html(tera.render("index.html", &Context::new()).unwrap())
        });

    // Aggregation of the above routes
    let routes = warp::any()
        .and(static_files_route)
        .or(invalid_static_files_route)
        .or(dynamic_route);

    warp::serve(routes).run(bind_address).await;
}
