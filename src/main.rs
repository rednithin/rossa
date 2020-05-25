use clap::{self, Clap};
use dotenv;
use log;
use mime_guess;
use pretty_env_logger;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rust_embed::RustEmbed;
use std::{net::SocketAddr, str, sync::Arc};
use tera::{Context, Tera};
use tokio::fs;
use warp::{http::HeaderValue, path::FullPath, reply::Response, Filter, Rejection, Reply};

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

fn with_tera(
    tera: Arc<Tera>,
) -> impl Filter<Extract = (Arc<Tera>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tera.clone())
}

fn append_frag(text: &mut String, frag: &mut String) {
    if !frag.is_empty() {
        let encoded = frag
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|ch| u8::from_str_radix(&ch.iter().collect::<String>(), 16).unwrap())
            .collect::<Vec<u8>>();
        text.push_str(&std::str::from_utf8(&encoded).unwrap());
        frag.clear();
    }
}

fn url_decode(text: &str) -> String {
    let mut output = String::new();
    let mut encoded_ch = String::new();
    let mut iter = text.chars();
    while let Some(ch) = iter.next() {
        if ch == '%' {
            encoded_ch.push_str(&format!("{}{}", iter.next().unwrap(), iter.next().unwrap()));
        } else {
            append_frag(&mut output, &mut encoded_ch);
            output.push(ch);
        }
    }
    append_frag(&mut output, &mut encoded_ch);
    output
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

    // Generating prefix for static files randomly.
    let files_prefix: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
    log::info!("The randomly generated files prefix is {:?}.", files_prefix);

    // Different types of Routes.
    let favicon_route = warp::path("favicon.ico").map(|| serve_asset("favicon.ico").unwrap());

    let files_route = warp::path(files_prefix.clone()).and(warp::fs::dir("."));

    let invalid_files_route = warp::path(files_prefix.clone())
        .and(with_tera(tera.clone()))
        .map(|tera: Arc<Tera>| {
            let mut context = Context::new();
            context.insert("message", "The file you are searching for doesn't exist");
            warp::reply::html(tera.render("404.html", &context).unwrap())
        });

    let dynamic_route = warp::path::full()
        .and(with_tera(tera.clone()))
        .and(warp::any().map(move || files_prefix.clone()))
        .and_then(
            |path: FullPath, tera: Arc<Tera>, files_prefix: String| async move {
                let path = url_decode(path.as_str());
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

                    log::info!("The path is {:?}", path);
                    log::info!("Directories: {:?}", directories);
                    log::info!("Files: {:?}", files);

                    let mut context = Context::new();
                    context.insert("files", &files);
                    context.insert("directories", &directories);

                    context.insert("files_prefix", &files_prefix);

                    Ok(warp::reply::html(
                        tera.render("index.html", &context).unwrap(),
                    ))
                } else {
                    log::error!("Files not found");
                    log::error!("The path is {:?}", path);
                    Err(warp::reject::not_found())
                }
            },
        );

    // Aggregation of the above routes.
    let routes = warp::any()
        .and(favicon_route)
        .or(files_route)
        .or(invalid_files_route)
        .or(dynamic_route);

    warp::serve(routes).run(bind_address).await;
}
