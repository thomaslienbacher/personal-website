use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use chrono::prelude::*;

fn main() {
    let index_contents = include_str!("./res/index.html");
    let local = Local::now();
    let index_final = index_contents.replace("{date}", &local.format("%a %b %e %T %Y").to_string());
    let mut index_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./static/index.html").unwrap();
    index_file.write_all(index_final.as_bytes()).unwrap();
}