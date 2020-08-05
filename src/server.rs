use warp::Filter;
use serde::{Serialize, Deserialize};
use geojson::{GeoJson, quick_collection};
use geo_types::{GeometryCollection,Geometry};
use nanoid;
use crate::collection::Collection;
use std::sync::Arc;
use std::sync::Mutex;
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
    let network = [0, 0, 0, 0];
    let port = 6985;
    let mut c = Collection::new();
    let mut c = Arc::new(Mutex::new(c));
    let mut c2 = Arc::clone(&c);
    let routes = warp::any()
        .and(warp::path("index"))
        .and(warp::body::content_length_limit(4196 * 16))
        .and(warp::body::json())
        .map(move |body: IndexRequestItem| {
            let id = match body.id {
                Some(res) => res,
                None => nanoid::nanoid!(),
            };

            c.lock().unwrap().add(id.clone(), body.geojson);

            return id;
        });
    let routes = warp::post().and(routes).or(
        warp::get().and(warp::path("nearby"))
        .and(warp::query::<SearchPoint>())
        .map(move |s: SearchPoint| {
            let n = c2.lock().unwrap().nearest(s.long, s.lat);
            Response::builder()
                .header("Content-Type", "application/json")
                .body(format!("{{\"id\": {}, \"geojson\": {}}}", n.0, n.1))
        })
    );
    warp::serve(routes).run((network, port)).await
}