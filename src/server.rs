use geojson::GeoJson;
use serde::{Deserialize, Serialize};

use crate::collection::Storage;
use crate::config::{get_conf, WELCOME_MESSAGE};
use colored::*;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use warp::http::Response;

fn backup(storage: Arc<RwLock<Storage>>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(get_conf().save_interval));
        interval.tick().await;
        loop {
            interval.tick().await;
            storage.read().save_to_file();
        }
    })
}

type DB = Arc<RwLock<Storage>>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct IndexRequestItem {
    id: Option<String>,
    geojson: GeoJson,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SearchPoint {
    long: f64,
    lat: f64,
}

mod routes {
    use super::*;
    use warp::Filter;

    fn with_storage(s: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || Arc::clone(&s))
    }

    pub fn all(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        add_geojson(storage.clone())
            .or(nearby(storage.clone()))
            .or(intersect(storage.clone()))
            .or(save(storage.clone()))
            .or(get_by_key(storage.clone()))
            .or(list(storage.clone()))
            .or(drop(storage.clone()))
            .or(remove_by_id(storage))
    }

    pub fn add_geojson(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(with_storage(storage))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::body::content_length_limit(4196 * 16))
            .and(warp::body::json())
            .map(|storage: DB, collection: String, body: IndexRequestItem| {
                let id = match body.id {
                    Some(res) => res,
                    None => nanoid::nanoid!(),
                };
                let col = storage.read().get(&collection);
                if let Some(col) = col {
                    col.write().add(id.clone(), body.geojson);
                } else {
                    storage.write().create(collection).write().add(id.clone(), body.geojson);
                }

                id
            })
    }

    pub fn nearby(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(with_storage(storage))
            .and(warp::path::param::<String>())
            .and(warp::path("nearby"))
            .and(warp::query::<SearchPoint>())
            .map(|storage: DB, collection: String, s: SearchPoint| {
                let col = storage.read().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().nearest(s.long, s.lat);
                }
                if let Some(n) = n {
                    Response::builder()
                        .header("Content-Type", "application/json")
                        .body(format!("{{\"id\": \"{}\", \"geojson\": {}}}", n.0, n.1))
                } else {
                    Response::builder().status(404).body("not found".to_owned())
                }
            })
    }

    pub fn intersect(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(with_storage(storage))
            .and(warp::path::param::<String>())
            .and(warp::path("intersect"))
            .and(warp::query::<SearchPoint>())
            .map(|storage: DB, collection: String, s: SearchPoint| {
                let col = storage.read().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().intersect(s.long, s.lat);
                }
                if let Some(n) = n {
                    Response::builder()
                        .header("Content-Type", "application/json")
                        .body(format!("{{\"id\": \"{}\", \"geojson\": {}}}", n.0, n.1))
                } else {
                    Response::builder().status(404).body("not found".to_owned())
                }
            })
    }

    pub fn save(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::put()
            .and(with_storage(storage))
            .and(warp::path("save"))
            .map(|storage: DB| {
                storage.read().save_to_file();
                Response::builder().status(200).body("OK".to_owned())
            })
    }

    pub fn get_by_key(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(with_storage(storage))
            .and(warp::path::param::<String>())
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .map(|storage: DB, collection: String, id: String| {
                let col = storage.read().get(&collection);
                let mut n = None;
                if let Some(col) = col {
                    n = col.read().get(&id);
                }
                if let Some(n) = n {
                    Response::builder()
                        .header("Content-Type", "application/json")
                        .body(format!("{{\"id\": \"{}\", \"geojson\": {}}}", id, n))
                } else {
                    Response::builder().status(404).body("not found".to_owned())
                }
            })
    }

    pub fn list(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(with_storage(storage))
            .and(warp::path::end())
            .map(|storage: DB| warp::reply::json(&storage.read().list()))
    }

    pub fn drop(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::delete()
            .and(with_storage(storage))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .map(|storage: DB, collection: String| {
                storage.write().remove(collection);
                "OK"
            })
    }

    pub fn remove_by_id(storage: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::delete()
            .and(with_storage(storage))
            .and(warp::path::param::<String>())
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .map(|storage: DB, collection: String, id: String| {
                let col = storage.read().get(&collection);
                if let Some(col) = col {
                    col.write().remove(id);
                }
                "OK"
            })
    }
}

pub async fn serve() {
    println!("{}", WELCOME_MESSAGE);
    let s = match Storage::load_from_file() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}",
                format!("Unable to load data file: {}. Creating an empty server", e.to_string()).yellow()
            );
            Storage::new()
        }
    };
    let storage = Arc::new(RwLock::new(s));
    backup(Arc::clone(&storage));
    let (host, port) = (get_conf().host, get_conf().port);
    println!("{}", format!("Server running on {}:{}", host.to_string(), port).green());
    warp::serve(routes::all(storage)).run((host, port)).await
}
