use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Asset;

#[derive(RustEmbed)]
#[folder = "templates"]
pub struct Template;
