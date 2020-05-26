use clap::{self, Clap};

/// A SimpleHTTPServer clone written in Rust.
/// This is also inspired by gossa - https://github.com/pldubouilh/gossa.
#[derive(Clap)]
#[clap(
    version = "0.1.1",
    author = "P G Nithin Reddy <reddy.nithinpg@gmail.com>"
)]
pub struct Opts {
    /// `address` must be of the form <IP>:<Port>
    #[clap(short, long, default_value = "0.0.0.0:8888")]
    pub address: String,
}
