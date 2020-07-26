use warp::Filter;
use serde::{Serialize, Deserialize};
use geojson::GeoJson;
use nanoid;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct IndexItem {
    id: Option<String>,
    geojson: GeoJson,
}

pub async fn serve() {
    let network = [0, 0, 0, 0];
    let port = 6985;
    let routes = warp::post()
        .and(warp::path("index"))
        .and(warp::body::content_length_limit(4196 * 16))
        .and(warp::body::json())
        .map(|mut body: IndexItem| {
            match body.id {
                Some(res) => res,
                None => nanoid::nanoid!(4),
            }
        });
    warp::serve(routes).run((network, port)).await
}