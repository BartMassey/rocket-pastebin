// Copyright © 2016 Bart Massey

// Rocket pastebin demo

#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate rand;
extern crate serde;
#[macro_use] extern crate serde_derive;

use std::io;
use std::io::BufRead;
use std::path::Path;
use std::collections::HashMap;

use rocket::Data;
use rocket::response::NamedFile;
use rocket_contrib::Template;

mod paste_id;
use paste_id::PasteID;

#[derive(Serialize)]
struct PasteInfo {
    paste_id: String,
    snippet: String,
    ellipsis: bool
}

#[get("/")]
fn index() -> io::Result<Template> {
    let dirents = std::fs::read_dir("upload/").unwrap();
    let pastes = dirents.map(|d| {
        let paste_id = d.unwrap().file_name().into_string().unwrap();
        let mut paste_filename = "upload/".to_string();
        paste_filename.push_str(&paste_id);
        let paste_file = NamedFile::open(&paste_filename).unwrap();
        let mut paste_reader = io::BufReader::new(paste_file);
        let mut paste_line = String::new();
        let paste_len = paste_reader.read_line(&mut paste_line).unwrap();
        let (ellipsis, snippet) =
            if paste_len < 24 {
                (false, paste_line)
            } else {
                let result = (&paste_line)[0..24].to_string();
                (true, result)
            };
        PasteInfo { paste_id: paste_id, snippet: snippet, ellipsis: ellipsis }
    }).collect::<Vec<PasteInfo>>();
    let mut context: HashMap<&str, _> = HashMap::new();
    context.insert("pastes", &pastes);
    Ok(Template::render("index", &context))
}

#[post("/", data = "<paste>")]
fn upload(paste: Data) -> io::Result<String> {
    let paste_id = PasteID::new(8);
    let path = Path::new("upload/").join(paste_id.to_string());
    paste.stream_to_file(path)?;
    Ok(format!("http://localhost:8000/{}\n", paste_id))
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Option<NamedFile> {
    let filename = format!("upload/{id}", id = id);
    NamedFile::open(&filename).ok()
}

fn main() {
    let rocket = rocket::ignite();
    let rocket = rocket.mount("/", routes![index, upload, retrieve]);
    rocket.launch()
}
