#![feature(int_to_from_bytes)]
#![feature(tool_lints)]
#![allow(proc_macro_derive_resolution_fallback)] // FIXME Rust 2018 / Diesel 1.4+
#[macro_use]
extern crate diesel; // FIXME +++

use bytes::buf::Buf;
use chrono::{Duration, Utc};
use crate::db::Database;
use crate::model::Item;
use lazy_static::lazy_static;
use mime::{Mime, BOUNDARY};
use multipart::server::Multipart;
use std::env;
use std::net::ToSocketAddrs;
use std::time::Duration as StdDuration;
use tokio::prelude::*;
use tokio::timer::Interval;
use warp::{body::FullBody, http::Response, http::StatusCode, Filter, Rejection, Reply};

mod db;
mod model;
mod schema;

lazy_static! {
    static ref DB: Database = Database::connect(&env::var("DATABASE_URL").expect("DATABASE_URL"));
    static ref SALT: Vec<u8> = env::var("ZERO_SALT").expect("ZERO_SALT").into_bytes();
    static ref URL: String = env::var("ZERO_URL").expect("ZERO_URL");
}

fn main() {
    dotenv::dotenv().ok();
    log_panics::init();
    pretty_env_logger::init();

    // `GET /`
    let index = warp::get2()
        .and(warp::path::index())
        .map(|| include_str!("../templates/post.html"));

    // `POST /`
    let create = warp::post2()
        .and(warp::path::index())
        .and(warp::header::<Mime>("Content-Type"))
        .and(warp::body::concat())
        .and_then(post);

    // `GET /:id`
    let read = warp::get2().and(warp::path::param()).and_then(get);

    // `PUT /:id`
    let update = warp::put2()
        .and(warp::path::param())
        .and(warp::header::<Mime>("Content-Type"))
        .and(warp::body::concat())
        .and_then(put);

    // `DELETE /:id`
    let delete = warp::delete2().and(warp::path::param()).and_then(delete);

    let routes = index.or(create).or(read).or(update).or(delete);
    let routes = routes.with(warp::log("zero"));

    let timer = {
        Interval::new_interval(StdDuration::from_secs(60))
            .map_err(|e| log::error!("server error: {}", e))
            .for_each(move |_| {
                match DB.sunset_items() {
                    Ok(0) => (),
                    Ok(1) => log::info!("sunsetted 1 paste"),
                    Ok(c) => log::info!("sunsetted {} pastes", c),
                    Err(e) => log::error!("server error: {}", e),
                };

                Ok(())
            })
    };

    let sockets = std::env::var("LISTEN_ADDR")
        .expect("LISTEN_ADDR")
        .to_socket_addrs()
        .expect("parse socket addresses");

    tokio::run(future::lazy(move || {
        tokio::spawn(timer);

        sockets.for_each(|sock| {
            let (addr, fut) = warp::serve(routes).bind_ephemeral(sock);
            log::info!("listening on {}", addr);
            tokio::spawn(fut);
        });

        Ok(())
    }));
}

#[allow(clippy::needless_pass_by_value)]
fn post(mime: Mime, body: FullBody) -> Result<impl Reply, Rejection> {
    let boundary = mime
        .get_param(BOUNDARY)
        .ok_or_else(warp::reject::bad_request)?
        .as_str();

    let id = DB.get_next_id()?;
    let data = Multipart::with_body(body.reader(), boundary);
    let item = Item::new_with_id(id).read_multipart_body(data)?;

    DB.add_item(&item)?;

    Ok(format!(
        "long: +{}\nshort: {}\nsize: {}\n\n{}",
        item.digest,
        item.label,
        item.content.len(),
        item.url()
    ))
}

#[allow(clippy::needless_pass_by_value)]
fn get(id: String) -> Result<impl Reply, Rejection> {
    let item = DB.get_item(&id)?;

    if item.destruct {
        let duration = Utc::now().signed_duration_since(item.timestamp.unwrap());
        if duration > Duration::seconds(10) {
            DB.delete_item(&item)?;
        }
    }

    if item.is_url {
        Ok(Response::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .header("Location", item.content.as_slice())
            .header("Content-Type", item.mimetype)
            .header(
                "Content-Disposition",
                format!("inline; filename=\"{}\"", item.filename),
            )
            .body([b"Redirecting to ", item.content.as_slice()].concat())
            .map_err(|_| warp::reject::server_error())?)
    } else {
        Ok(Response::builder()
            .header("Content-Type", item.mimetype)
            .header(
                "Content-Disposition",
                format!("inline; filename=\"{}\"", item.filename),
            )
            .body(item.content)
            .map_err(|_| warp::reject::server_error())?)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn put(id: String, mime: Mime, body: FullBody) -> Result<impl Reply, Rejection> {
    let boundary = mime
        .get_param(BOUNDARY)
        .ok_or_else(warp::reject::bad_request)?
        .as_str();
    let data = Multipart::with_body(body.reader(), boundary);
    let item = DB.get_item(&id)?.read_multipart_body(data)?;

    DB.update_item(&item)?;

    Ok(format!(
        "long: +{}\nshort: {}\nsize: {}\n\n{}",
        item.digest,
        item.label,
        item.content.len(),
        item.url()
    ))
}

#[allow(clippy::needless_pass_by_value)]
fn delete(id: String) -> Result<impl Reply, Rejection> {
    DB.delete_item_by_digest(&id)?;
    Ok("Deleted.")
}
