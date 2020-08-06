use warp::Filter;
use serde::{Serialize, Deserialize};
use geojson::GeoJson;
use nanoid;
use crate::collection::Storage;
use crate::config::{get_conf, WELCOME_MESSAGE};
use colored::*;
use std::sync::Arc;
use std::sync::RwLock;
use warp::http::Response;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct IndexRequestItem {
    id: Option<String>,
    geojson: GeoJson,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SearchPoint {
    long: f64,
    lat: f64
}

pub async fn serve() {
    println!("{}", WELCOME_MESSAGE);
    let s = match Storage::load_from_file() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}",
                format!("Unable to load data file: {}. Creating an empty server", e.to_string()).yellow()
            );
            Storage::new()
        }
    };
    let storage = Arc::new(RwLock::new(s));
    let storage1 = Arc::clone(&storage);
    let storage2 = Arc::clone(&storage);
    let storage3 = Arc::clone(&storage);
    let routes = warp::any()
        .and(warp::path::param::<String>())
        .and(warp::body::content_length_limit(4196 * 16))
        .and(warp::body::json())
        .map(move |collection: String, body: IndexRequestItem| {
            let id = match body.id {
                Some(res) => res,
                None => nanoid::nanoid!(),
            };
            let col = storage1.read().unwrap().get(&collection);
            if let Some(col) = col {
                col.write().unwrap().add(id.clone(), body.geojson);
            } else {
                storage1.write().unwrap().create(collection)
                    .write().unwrap().add(id.clone(), body.geojson);
            }

            return id;
        });
    let routes = warp::post().and(routes).or(
    warp::get()
            .and(warp::path::param::<String>())
            .and(warp::path("nearby"))
            .and(warp::query::<SearchPoint>())
            .map(move |collection: String, s: SearchPoint| {
                let col = storage2.read().unwrap().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().unwrap().nearest(s.long, s.lat);
                }
                if let Some(n) = n {
                    Response::builder()
                        .header("Content-Type", "application/json")
                        .body(format!("{{\"id\": \"{}\", \"geojson\": {}}}", n.0, n.1))
                } else {
                    Response::builder()
                        .status(404)
                        .body(format!("not found"))
                }
            })
    ).or(
warp::put()
        .and(warp::path("save"))
        .map(move || {
            storage3.read().unwrap().save_to_file();
            Response::builder()
                .status(200)
                .body(format!("OK"))
        })
    );
    let (host, port) = (get_conf().host, get_conf().port);
    println!("{}", format!("Server running on {}:{}", host.to_string(), port).green());
    warp::serve(routes).run((host, port)).await
}