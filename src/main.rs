// Copyright © 2016 Bart Massey

// Rocket pastebin demo

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rand;

use std::io;
use std::path::Path;
use rocket::Data;
use rocket::response::NamedFile;

mod paste_id;
use paste_id::PasteID;

#[get("/")]
fn index() -> &'static str {
    "Welcome to Rocket demo pastebin!"
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
