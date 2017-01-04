// Copyright © 2016 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file COPYING in the source
// distribution of this software for license terms.

// Rocket pastebin demo

#![feature(plugin)]
#![feature(plugin, custom_derive)]
#![feature(proc_macro)]
#![plugin(rocket_codegen)]

static UPLOAD: &'static str = "upload";
static NID: usize = 8;

extern crate rocket;
extern crate rocket_contrib;
extern crate rand;
extern crate serde;
#[macro_use] extern crate serde_derive;

use std::io;
use std::io::{Read, BufRead, Write};
use std::path::Path;
use std::collections::HashMap;
use std::fs::{File, create_dir, metadata, remove_file};

use rocket::Data;
use rocket::response::NamedFile;
use rocket_contrib::Template;
use rocket::request::Form;
use rocket::response::Redirect;

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
    let dirents = std::fs::read_dir(UPLOAD)
                  .expect("could not open upload directory");
    let pastes = dirents.map(|d| {
        let paste_id = d.unwrap().file_name().into_string().unwrap();
        let paste_filename = Path::new(UPLOAD).join(&paste_id);
        let paste_file = NamedFile::open(&paste_filename)
                         .expect("could not open paste file");
        let mut paste_reader = io::BufReader::new(paste_file);
        let mut paste_line = String::new();
        let paste_len = paste_reader.read_line(&mut paste_line)
                        .expect("could not read first line of paste");
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
    let paste_id = PasteID::new(NID);
    let path = Path::new(UPLOAD).join(&paste_id.to_string());
    paste.stream_to_file(path)?;
    Ok(format!("/{}\n", paste_id))
}

fn open_paste(id: &PasteID) -> io::Result<NamedFile> {
    let filename = Path::new(UPLOAD).join(id.to_string());
    NamedFile::open(&filename)
}

fn create_paste(id: &PasteID) -> io::Result<File> {
    let filename = Path::new(UPLOAD).join(id.to_string());
    File::create(&filename)
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Option<NamedFile> {
    open_paste(&id).ok()
}

#[get("/new")]
fn edit_new() -> io::Result<Redirect> {
    let paste_id = PasteID::new(NID);
    let f = try!(create_paste(&paste_id));
    drop(f);
    Ok(Redirect::to(&format!("/edit/{}", paste_id.to_string())))
}

#[get("/edit/<id>")]
fn make_edit(id: PasteID) -> io::Result<Template> {
    let paste = open_paste(&id)?;
    let mut paste_reader = io::BufReader::new(paste);
    let mut paste_contents = String::new();
    paste_reader.read_to_string(&mut paste_contents)?;
    let paste_id = id.to_string();
    let paste_rows = format!("{}", 100);
    let mut context: HashMap<&str, _> = HashMap::new();
    context.insert("contents", &paste_contents);
    context.insert("paste_id", &paste_id);
    context.insert("paste_rows", &paste_rows);
    Ok(Template::render("edit", &context))
}

#[derive(FromForm)]
struct EditForm {
    paste: String
}

#[post("/edit/<id>", data = "<form>")]
fn accept_edit(id: PasteID, form: Form<EditForm>) -> io::Result<Redirect> {
    let edit = form.get();
    let paste = create_paste(&id)?;
    let mut paste_writer = io::BufWriter::new(paste);
    let paste_contents =
        edit.paste.chars().map(|c|{c as u8}).collect::<Vec<u8>>();
    paste_writer.write_all(paste_contents.as_slice())?;
    paste_writer.flush()?;
    Ok(Redirect::to("/"))
}

#[get("/delete/<id>")]
fn delete(id: PasteID) -> io::Result<Redirect> {
    let filename = Path::new(UPLOAD).join(id.to_string());
    try!(remove_file(&filename));
    Ok(Redirect::to("/"))
}

fn ok_dir<P: AsRef<Path>>(path: P) -> bool {
    match metadata(path.as_ref()) {
        Ok(md) => md.is_dir(),
        _ => false
    }
}

fn check_upload_dir() {
    if !ok_dir(UPLOAD) {
        create_dir(UPLOAD).expect("could not create upload directory");
    }
}

fn main() {
    check_upload_dir();
    let rocket = rocket::ignite();
    let rocket = rocket.mount("/", routes![index, upload, edit_new, retrieve,
                                           make_edit, accept_edit, delete]);
    rocket.launch()
}


#[cfg(test)]
mod test {
    extern crate scraper;

    use super::rocket;
    use rocket::testing::MockRequest;
    use rocket::http::{Status, Method};
    use self::scraper::{Html, Selector};

    #[test]
    fn index() {
        let rocket = rocket::ignite().mount("/", routes![super::index]);

        let mut req = MockRequest::new(Method::Get, "/");
        let mut response = req.dispatch_with(&rocket);

        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap_or(String::from(""));
        assert_has_text(&body_str, &"Pastebin!");
        assert_has_selector(&body_str, &"ul");
        assert_no_selector(&body_str, &"ul li");
    }

    fn assert_has_text(body: &str, expected: &str) {
      let body_html = Html::parse_fragment(body);
      let selector = Selector::parse("*").unwrap();
      let elements = body_html.select(&selector);
      let texts = elements.map(|e| e.text().collect::<Vec<_>>().concat());
      let actual = texts.fold(String::new(), |acc, s| format!("{}:{}", acc, s));
      assert!(&actual.contains(expected));
    }

    fn assert_has_selector(body: &str, selector_text: &str) {
      let body_html = Html::parse_fragment(body);
      let selector = Selector::parse(selector_text).unwrap();
      let count = body_html.select(&selector).count();
      assert!(count > 0);
    }

    fn assert_no_selector(body: &str, selector_text: &str) {
      let body_html = Html::parse_fragment(body);
      let selector = Selector::parse(selector_text).unwrap();
      let count = body_html.select(&selector).count();
      assert_eq!(0, count);
    }
}
