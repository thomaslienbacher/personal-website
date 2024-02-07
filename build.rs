use std::fs::{File, OpenOptions};
use std::io::Write;
use chrono::prelude::*;

fn main() {
    let index_contents = include_str!("./res/index.html");
    let local = Local::now();
    let index_metadata = File::open("./res/index.html").unwrap().metadata().unwrap();
    let modified: DateTime<Local> = index_metadata.modified().unwrap().into();

    let index_final = index_contents
        .replace("{date}", &modified.format("%a %b %e %Y %T").to_string())
        .replace("{compile_date}", &local.format("%a %b %e %Y %T").to_string());

    let mut index_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./static/index.html").unwrap();
    index_file.write_all(index_final.as_bytes()).unwrap();
}