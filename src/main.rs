use async_std::sync::Arc;
use dotenv;
use log;
use pretty_env_logger;
use tera::{Context, Tera};
use warp::Filter;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Something wrongh with .env file");
    pretty_env_logger::init();

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

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
