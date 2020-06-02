use super::embed::Template;
use std::str;
use tera::Tera;

pub fn fetch() -> Tera {
    let mut tera = Tera::default();

    for file in Template::iter() {
        let file_name = file.as_ref();
        let file_contents = Template::get(file_name).unwrap();
        let content = str::from_utf8(file_contents.as_ref()).unwrap();
        tera.add_raw_template(file_name, content)
            .expect("Failed to add raw template");
    }

    tera
}
