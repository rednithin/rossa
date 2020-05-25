use clap::Clap;
use dotenv;
use log;
use mime_guess;
use pretty_env_logger;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::env;
use std::{net::SocketAddr, str, sync::Arc};
use tera::{Context, Tera};
use tokio::fs;
use warp::{http::HeaderValue, path::FullPath, reply::Response, Filter, Rejection, Reply};

mod cli;
mod embed;
mod templates;
mod util;

use embed::Asset;

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

fn with_cloneable<T: Clone + std::marker::Send>(
    t: T,
) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || t.clone())
}

#[tokio::main(core_threads = 1)]
async fn main() {
    dotenv::dotenv().unwrap_or_default();
    if let Err(_) = dotenv::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let opts = cli::Opts::parse();
    let bind_address: SocketAddr = opts.address.parse().expect("Invalid Bind Address");

    // Loading Tera templates.
    let tera = Arc::new(templates::fetch_templates());

    // Generating prefix for static files randomly.
    let files_prefix: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
    log::info!("The randomly generated files prefix is {:?}.", files_prefix);

    // Different types of Routes.
    let favicon_route = warp::any()
        .and(warp::get())
        .and(warp::path("favicon.ico"))
        .map(|| serve_asset("favicon.ico").unwrap());

    let files_route = warp::any()
        .and(warp::get())
        .and(warp::path(files_prefix.clone()))
        .and(warp::fs::dir("."));

    let invalid_files_route = warp::any()
        .and(warp::get())
        .and(warp::path(files_prefix.clone()))
        .and(with_cloneable(tera.clone()))
        .map(|tera: Arc<Tera>| {
            let mut context = Context::new();
            context.insert("message", "The file you are searching for doesn't exist");
            warp::reply::html(tera.render("404.tera", &context).unwrap())
        });

    let invalid_path_route = warp::any()
        .and(warp::get())
        .and(warp::path::full())
        .and(with_cloneable(tera.clone()))
        .map(|path: FullPath, tera: Arc<Tera>| {
            let mut context = Context::new();
            context.insert(
                "message",
                &format!("The path '.{}' doesn't exist", path.as_str()),
            );
            warp::reply::html(tera.render("404.tera", &context).unwrap())
        });

    let dynamic_route = warp::any()
        .and(warp::get())
        .and(warp::path::full())
        .and(with_cloneable(tera.clone()))
        .and(with_cloneable(files_prefix.clone()))
        .and_then(
            |path: FullPath, tera: Arc<Tera>, files_prefix: String| async move {
                let path = util::url_decode(path.as_str());
                if let Ok(mut entries) = fs::read_dir(".".to_string() + path.as_str()).await {
                    log::info!("entries {:?}", entries);

                    let mut directories = vec![];
                    let mut files = vec![];

                    while let Ok(entry) = entries.next_entry().await {
                        if let Some(entry) = entry {
                            if let Ok(entry_type) = entry.file_type().await {
                                if entry_type.is_dir() {
                                    directories.push((entry.file_name().into_string().unwrap(), {
                                        let x = entry.path();
                                        x.to_str().unwrap()[1..].to_string()
                                    }));
                                } else {
                                    files.push((entry.file_name().into_string().unwrap(), {
                                        let x = entry.path();
                                        x.to_str().unwrap()[1..].to_string()
                                    }));
                                }
                            }
                        } else {
                            break;
                        }
                    }

                    if path.as_str() != "/" {
                        let mut tokens: Vec<&str> = path.as_str().split("/").collect();
                        tokens.pop();
                        let parent_path = if tokens.len() == 1 {
                            "/".to_string()
                        } else {
                            tokens.join("/")
                        };
                        log::info!("Parent path {:?}", parent_path);
                        directories.push((String::from(".."), parent_path));
                    }

                    directories.sort_unstable();
                    files.sort_unstable();

                    log::info!("The path is {:?}", path);
                    log::info!("Directories: {:?}", directories);
                    log::info!("Files: {:?}", files);

                    let mut context = Context::new();
                    context.insert("files", &files);
                    context.insert("directories", &directories);
                    context.insert("files_prefix", &files_prefix);
                    context.insert("current_dir", &path);

                    Ok(warp::reply::html(
                        tera.render("index.tera", &context).unwrap(),
                    ))
                } else {
                    log::error!("Files not found");
                    log::error!("The path is {:?}", path);
                    Err(warp::reject::not_found())
                }
            },
        )
        .or(invalid_path_route);

    // Aggregation of the above routes.
    let routes = warp::any()
        .and(favicon_route)
        .or(files_route)
        .or(invalid_files_route)
        .or(dynamic_route);

    warp::serve(routes).run(bind_address).await;
}
