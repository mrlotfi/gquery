use std::collections::HashMap;
use geo_types::{GeometryCollection,Geometry, Point, LineString};
use spade::rtree::RTree;
use spade::{SpatialObject, BoundingRect};
use serde::{Serialize, Deserialize};
use geo::prelude::*;
use geo::algorithm::bounding_rect::BoundingRect as br;
use geojson::{GeoJson, quick_collection};
use std::sync::{RwLock, Arc};
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::time::Instant;


struct IndexItem {
    id: String,
    geom: Geometry<f64>,
}

impl SpatialObject for IndexItem {
    type Point = [f64; 2];

    fn mbr(&self) -> BoundingRect<Self::Point> {
        let rect = self.geom.bounding_rect().unwrap();
        BoundingRect::from_corners(
            &[rect.min().x, rect.min().y],
            &[rect.max().x, rect.max().y],
        )
    }

    fn distance2(&self, point: &Self::Point) -> f64 {
        match &self.geom {
            Geometry::Point(p) => {
                p.euclidean_distance(&Point::new(point[0], point[1]))
            },
            Geometry::Polygon(p) => {
                p.euclidean_distance(&Point::new(point[0], point[1]))
            },
            Geometry::Line(p) => {
                p.euclidean_distance(&Point::new(point[0], point[1]))
            },
            _ => panic!("Shouldnt reach here"),
        }
    }

    fn contains(&self, point: &Self::Point) -> bool {
        self.geom.contains(&Point::new(point[0], point[1]))
    }
}


pub struct Storage {
    collections: HashMap<String, Arc<RwLock<Collection>>>,
}


#[derive(Debug, Deserialize, Serialize)]
struct Saved {
    items: HashMap<String, HashMap<String, String>>
}


impl Storage {
    pub fn new() -> Self {
        Storage {
            collections: HashMap::new(),
        }
    }

    pub fn save_to_file(&self) {
        let mut saved = Saved {
            items: HashMap::new()
        };
        for (col, val) in &self.collections {
            saved.items.insert(col.clone(), HashMap::new());
            for (key, geojson_str) in &val.read().unwrap().objects {
                saved.items.get_mut(col).unwrap().insert(key.clone(), geojson_str.clone());
            }
        }
        serde_json::to_writer(&File::create("data.json").unwrap(), &saved).unwrap();
    }

    pub fn load_from_file() -> Self {
        let start = Instant::now();
        let mut s = Self::new();
        let saved: Saved = serde_json::from_reader(BufReader::new(File::open("data.json").unwrap())).unwrap();
        let mut n: usize = 0;
        for (col, val) in saved.items {
            let mut collection = Collection::new();
            for (id, geojson_str) in val {
                collection.add(id, GeoJson::from_str(&geojson_str).unwrap());
            }
            n += collection.objects.len();
            s.collections.insert(col, Arc::new(RwLock::new(collection)));
        }
        println!("Loaded {} items from saved db in {:.2} seconds", n, start.elapsed().as_secs_f32());
        s
    }

    pub fn get(&self, key: &String) -> Option<Arc<RwLock<Collection>>> {
        self.collections.get(key).map(|arc| {
            Arc::clone(arc)
        })
    }

    pub fn create(&mut self, key: String) -> Arc<RwLock<Collection>> {
        self.collections.insert(key.clone(), Arc::new(RwLock::new(Collection::new())));
        return Arc::clone(&self.collections[&key]);
    }
}


pub struct Collection {
    idx: RTree<IndexItem>,
    objects: HashMap<String, String>,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            idx: RTree::new(),
            objects: HashMap::new()
        }
    }

    pub fn add(&mut self, id: String, geojson: GeoJson) -> bool {
        let collection: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
        collection.into_iter().for_each(|geom| {
            match geom {
                Geometry::LineString(p) => {
                    self.index_linestring(&id, p);
                },
                Geometry::MultiLineString(p) => {
                    p.into_iter().for_each(|line_string| {
                        self.index_linestring(&id, line_string);
                    });
                },
                Geometry::MultiPolygon(p) => {
                    p.into_iter().for_each(|poly| {
                        self.idx.insert(IndexItem {
                            id: id.clone(),
                            geom: Geometry::Polygon(poly),
                        })
                    });
                },
                _ => {
                    self.idx.insert(IndexItem {
                        id: id.clone(),
                        geom,
                    })
                },
            }
        });
        self.objects.insert(id, geojson.to_string());
        self.nearest(2., 2.);
        true
    }

    fn index_linestring(&mut self, id: &String, p: LineString<f64>) {
        p.lines().into_iter().for_each(|sub_geom| {
            self.idx.insert(IndexItem {
                id: id.clone(),
                geom: Geometry::Line(sub_geom),
            })
        });
    }

    pub fn nearest(&self, long: f64, lat: f64) -> Option<(String, String)> {
        return self.idx
            .nearest_neighbor(&[long, lat])
            .map(|n| {
                (n.id.clone(), self.objects.get(&n.id).unwrap().clone())
            });
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use geo::{point, line_string, Geometry, polygon};
    #[test]
    fn test_add() {
        let p = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 1.),
            (x: 1., y: 1.),
            (x: 1., y: 0.),
            (x: 0., y: 0.),
        ];
        let poi = point!(x: 1.1, y:0.9);
        let d = p.euclidean_distance(&poi);
        println!("{}", d);
        // let mut c = Collection::new();
        // c.idx.insert(IndexItem {
        //     id: String::from("123"),
        //     geom: Geometry::LineString(line_string![
        //         (x: 0., y: 1.),
        //         (x: 200., y: 1.),
        //     ]),
        // });
        // c.idx.insert(IndexItem {
        //     id: String::from("456"),
        //     geom: Geometry::LineString(line_string![
        //         (x: 49., y: -1.5),
        //         (x: 51., y: -1.5),
        //     ]),
        // });
        // let d = c.idx.nearest_neighbor(&[50.0, 0.0]);
        // assert_eq!(1, 1);
    }
}