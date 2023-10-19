#![allow(clippy::no_effect_underscore_binding, clippy::ignored_unit_patterns)]
use clap::Parser;
use rocket::{
    error::ErrorKind,
    request::{self, FromRequest, Outcome, Request},
    response::Redirect,
    Error,
};
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::BufReader};

#[macro_use]
extern crate rocket;

struct RedirectPath(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RedirectPath {
    type Error = Option<()>;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let path = req.uri().path().to_string();
        let url_map = req
            .rocket()
            .state::<HashMap<String, String>>()
            .expect("Can't get the url_map");
        if let Some(p) = url_map.get(&path) {
            return Outcome::Success(RedirectPath(p.clone()));
        }
        Outcome::Forward(())
    }
}

#[get("/<_>")]
fn redirect(new_path: RedirectPath) -> Redirect {
    Redirect::to(new_path.0)
}

#[get("/<path>", rank = 2)]
fn hi(path: &str) -> String {
    format!("Hi! From {path}")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "input/url-map.json")]
    file_path: String,
}

#[derive(Debug, Deserialize)]
struct UrlMap {
    path: String,
    url: String,
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    // Get command line arguments
    let args = Args::parse();
    // Load JSON file
    let file = File::open(args.file_path).map_err(ErrorKind::Io)?;
    let reader = BufReader::new(file);
    // Deserialize JSON
    let deserialized_json: Vec<UrlMap> = serde_json::from_reader(reader).map_err(|_| {
        ErrorKind::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Can't open the JSON file",
        ))
    })?;
    // Convert to HashMap
    let url_map = deserialized_json
        .iter()
        .map(|url_map| (url_map.path.clone(), url_map.url.clone()))
        .collect::<HashMap<String, String>>();

    rocket::build()
        .mount("/", routes![hi, redirect])
        .manage(url_map)
        .launch()
        .await?;

    Ok(())
}
