use clap::{self, Clap};
use dotenv;
use log;
use mime_guess;
use pretty_env_logger;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rust_embed::RustEmbed;
use std::{net::SocketAddr, str, sync::Arc};
use tera::{Context, Tera};
use warp::{http::HeaderValue, reply::Response, Filter, Rejection, Reply};

/// A SimpleHTTPServer clone written in Rust.
/// This is also inspired by gossa - https://github.com/pldubouilh/gossa.
#[derive(Clap)]
#[clap(
    version = "1.0.0",
    author = "P G Nithin Reddy <reddy.nithinpg@gmail.com>"
)]
struct Opts {
    /// `address` must be of the form <IP>:<Port>
    #[clap(short, long, default_value = "0.0.0.0:8888")]
    address: String,
}

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

#[derive(RustEmbed)]
#[folder = "templates"]
struct Template;

fn serve_asset(path: &str) -> Result<impl Reply, Rejection> {
    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}

#[tokio::main(core_threads = 1)]
async fn main() {
    dotenv::dotenv().unwrap_or_default();
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();
    let bind_address: SocketAddr = opts.address.parse().expect("Invalid Bind Address");

    // Loading Tera templates.
    let mut tera = Tera::default();

    for file in Template::iter() {
        let file_name = file.as_ref();
        let file_contents = Template::get(file_name).unwrap();
        let content = str::from_utf8(file_contents.as_ref()).unwrap();
        tera.add_raw_template(file_name, content)
            .expect("Failed to add raw template");
    }

    let tera = Arc::new(tera);
    let tera1 = tera.clone();

    // Generating prefix for static files randomly.
    let files_prefix: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
    log::info!("The randomly generated files prefix is {:?}.", files_prefix);

    // Different types of Routes.
    let favicon_route = warp::path("favicon.ico").map(|| serve_asset("favicon.ico").unwrap());

    let files_route = warp::path(files_prefix.clone()).and(warp::fs::dir("."));

    let invalid_files_route = warp::path(files_prefix.clone())
        .and(warp::any().map(move || tera.clone()))
        .map(|tera: Arc<Tera>| {
            let mut context = Context::new();
            context.insert("message", "The file you are searching for doesn't exist");
            warp::reply::html(tera.render("404.html", &context).unwrap())
        });

    let dynamic_route = warp::path::full()
        .and(warp::any().map(move || tera1.clone()))
        .map(|path, tera: Arc<Tera>| {
            log::info!("The path is {:?}", path);
            warp::reply::html(tera.render("index.html", &Context::new()).unwrap())
        });

    // Aggregation of the above routes.
    let routes = warp::any()
        .and(favicon_route)
        .or(files_route)
        .or(invalid_files_route)
        .or(dynamic_route);

    warp::serve(routes).run(bind_address).await;
}
