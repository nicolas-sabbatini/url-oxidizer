#![allow(clippy::no_effect_underscore_binding, clippy::ignored_unit_patterns)]
use diesel::prelude::*;
use html_node::{html, text};
use rocket::{
    http::{ContentType, Status},
    request::{self, FromRequest, Outcome, Request},
    response::{Debug, Redirect},
    serde::{json::Json, Deserialize, Serialize},
    Error,
};
use rocket_sync_db_pools::{database, diesel};

#[macro_use]
extern crate rocket;

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

struct RedirectPath(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RedirectPath {
    type Error = Option<()>;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let db = req.guard::<Db>().await.expect("Unble to get DB connection");
        let path = req.uri().path().to_string();
        if let Ok(p) = db
            .run(|conn| {
                url_map::table
                    .filter(url_map::path.eq(path))
                    .select(url_map::url)
                    .first::<String>(conn)
            })
            .await
        {
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

#[post("/", data = "<map>")]
async fn create(map: Json<UrlMap>, db: Db) -> Result<Status> {
    db.run(move |conn| {
        diesel::insert_into(url_map::table)
            .values(&*map)
            .execute(conn)
    })
    .await?;
    Ok(Status::Created)
}

#[patch("/", data = "<map>")]
async fn update(map: Json<UrlMap>, db: Db) -> Result<Status> {
    let res = db
        .run(move |conn| {
            diesel::update(url_map::table)
                .filter(url_map::path.eq(map.path.clone()))
                .set(url_map::url.eq(map.url.clone()))
                .execute(conn)
        })
        .await?;
    if res == 0 {
        return Ok(Status::NotFound);
    }
    Ok(Status::NoContent)
}

#[database("url_map")]
struct Db(diesel::SqliteConnection);

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = url_map)]
struct UrlMap {
    path: String,
    url: String,
}

table! {
    url_map (path) {
        path -> Text,
        url -> Text,
    }
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    rocket::build()
        .attach(Db::fairing())
        .mount("/", routes![hi, redirect, create, update])
        .launch()
        .await?;

    Ok(())
}
