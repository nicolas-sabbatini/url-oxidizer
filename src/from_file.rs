#![allow(clippy::no_effect_underscore_binding, clippy::ignored_unit_patterns)]
use clap::Parser;
use html_node::{html, text};
use rocket::{
    error::ErrorKind,
    http::ContentType,
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
fn hi(path: &str) -> (ContentType, String) {
    let body = html! { <h1>Hi! From { text!(" {path}") } </h1> };
    (ContentType::HTML, body.to_string())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    json_path: Option<String>,
    #[arg(short, long)]
    yaml_path: Option<String>,
}

enum From {
    Json,
    Yaml,
}

#[derive(Debug, Deserialize)]
struct UrlMap {
    path: String,
    url: String,
}

fn parse_args() -> (String, From) {
    let args = Args::parse();
    if let Some(path) = args.json_path {
        return (path, From::Json);
    }
    if let Some(path) = args.yaml_path {
        return (path, From::Yaml);
    }
    panic!("Can't parse the arguments");
}

fn build_url_map_from_path(path: &str, from: &From) -> Result<HashMap<String, String>, Error> {
    let file = File::open(path).map_err(ErrorKind::Io)?;
    let reader = BufReader::new(file);
    let deserialized: Vec<UrlMap> = match from {
        From::Json => serde_json::from_reader(reader).map_err(|_| {
            ErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Can't open the JSON file",
            ))
        })?,
        From::Yaml => serde_yaml::from_reader(reader).map_err(|_| {
            ErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Can't open the YAML file",
            ))
        })?,
    };
    let url_map = deserialized
        .iter()
        .map(|url_map| (url_map.path.clone(), url_map.url.clone()))
        .collect::<HashMap<String, String>>();
    Ok(url_map)
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    let (path, from) = parse_args();
    let url_map = build_url_map_from_path(&path, &from)?;
    rocket::build()
        .mount("/", routes![hi, redirect])
        .manage(url_map)
        .launch()
        .await?;

    Ok(())
}
