use warp::Filter;
use serde::{Serialize, Deserialize};
use geojson::GeoJson;
use nanoid;
use crate::collection::Storage;
use crate::config::{get_conf, WELCOME_MESSAGE};
use colored::*;
use std::sync::Arc;
use parking_lot::RwLock;
use warp::http::Response;
use tokio::time::{interval, Duration};

fn backup(storage: Arc<RwLock<Storage>>) -> tokio::task::JoinHandle<()> {
    return tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(get_conf().save_interval));
        interval.tick().await;
        loop {
            interval.tick().await;
            storage.read().save_to_file();
        }
    });
}


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

fn add_geojson() -> warp::filters::BoxedFilter<(String, IndexRequestItem,)> {
    warp::post()
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::content_length_limit(4196 * 16))
        .and(warp::body::json())
        .boxed()
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
    let storage4 = Arc::clone(&storage);
    let storage5 = Arc::clone(&storage);
    backup(Arc::clone(&storage));
    let routes = add_geojson()
        .map(move |collection: String, body: IndexRequestItem| {
            let id = match body.id {
                Some(res) => res,
                None => nanoid::nanoid!(),
            };
            let col = storage1.read().get(&collection);
            if let Some(col) = col {
                col.write().add(id.clone(), body.geojson);
            } else {
                storage1.write().create(collection)
                    .write().add(id.clone(), body.geojson);
            }

            return id;
        });
    let routes = warp::post().and(routes).or(
    warp::get()
            .and(warp::path::param::<String>())
            .and(warp::path("nearby"))
            .and(warp::query::<SearchPoint>())
            .map(move |collection: String, s: SearchPoint| {
                let col = storage2.read().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().nearest(s.long, s.lat);
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
        warp::get()
            .and(warp::path::param::<String>())
            .and(warp::path("intersect"))
            .and(warp::query::<SearchPoint>())
            .map(move |collection: String, s: SearchPoint| {
                let col = storage5.read().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().intersect(s.long, s.lat);
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
            storage3.read().save_to_file();
            Response::builder()
                .status(200)
                .body(format!("OK"))
        })
    ).or(
        warp::get()
            .and(warp::path::param::<String>())
            .and(warp::path::param::<String>())
            .map(move |collection: String, id: String| {
                let col = storage4.read().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().get(&id);
                }
                if let Some(n) = n {
                    Response::builder()
                        .header("Content-Type", "application/json")
                        .body(format!("{{\"id\": \"{}\", \"geojson\": {}}}", id, n))
                } else {
                    Response::builder()
                        .status(404)
                        .body(format!("not found"))
                }
            })
    );
    let (host, port) = (get_conf().host, get_conf().port);
    println!("{}", format!("Server running on {}:{}", host.to_string(), port).green());
    warp::serve(routes).run((host, port)).await
}