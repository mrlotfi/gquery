use std::collections::HashMap;
use geo_types::{GeometryCollection,Geometry, Point, LineString};
use spade::rtree::RTree;
use spade::{SpatialObject, BoundingRect};
use serde::{Serialize, Deserialize};
use geo::prelude::*;
use geo::algorithm::bounding_rect::BoundingRect as br;
use geojson::{GeoJson, quick_collection};
use std::sync::Arc;
use parking_lot::RwLock;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::time::Instant;
use crate::config::get_conf;
use std::error::Error;


struct IndexItem {
    id: String,
    geom: Geometry<f64>,
}

impl PartialEq for IndexItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
    
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
        let start = Instant::now();
        let file_path = get_conf().data;
        let mut saved = Saved {
            items: HashMap::new()
        };
        let mut n: usize = 0;
        for (col, val) in &self.collections {
            saved.items.insert(col.clone(), HashMap::new());
            for (key, geojson_str) in &val.read().objects {
                saved.items.get_mut(col).unwrap().insert(key.clone(), geojson_str.clone());
                n += 1;
            }
        }
        bincode::serialize_into(&File::create(file_path).unwrap(), &saved).unwrap();
        println!("Saved {} items from current db in {:.2} seconds", n, start.elapsed().as_secs_f32());
    }

    pub fn load_from_file() -> Result<Self, Box<dyn Error>> {
        let file_path = get_conf().data;
        let start = Instant::now();
        let mut s = Self::new();
        let saved: Saved = bincode::deserialize_from(BufReader::new(File::open(file_path)?))?;
        let mut n: usize = 0;
        for (col, val) in saved.items {
            let mut collection = Collection::new();
            for (id, geojson_str) in val {
                collection.add(id, GeoJson::from_str(&geojson_str)?);
            }
            n += collection.objects.len();
            s.collections.insert(col, Arc::new(RwLock::new(collection)));
        }
        println!("Loaded {} items from saved db in {:.2} seconds", n, start.elapsed().as_secs_f32());
        Ok(s)
    }

    pub fn get(&self, key: &str) -> Option<Arc<RwLock<Collection>>> {
        self.collections.get(key).map(|arc| {
            Arc::clone(arc)
        })
    }

    pub fn create(&mut self, key: String) -> Arc<RwLock<Collection>> {
        self.collections.insert(key.clone(), Arc::new(RwLock::new(Collection::new())));
        Arc::clone(&self.collections[&key])
    }

    pub fn remove(&mut self, key: String) {
        self.collections.remove(&key);
    }

    pub fn list(&self) -> Vec<&String> {
        self.collections.keys().collect()
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

    pub fn get(&self, key: &str) -> Option<String> {
        self.objects.get(key).cloned()
    }

    pub fn remove(&mut self, id: String) {
        let t = self.objects.remove(&id);
        if t.is_none() {
            return;
        }
        let collection: GeometryCollection<f64> = quick_collection(&GeoJson::from_str(&t.unwrap()).unwrap()).unwrap();
        collection.into_iter().for_each(|geom| {
            match geom {
                Geometry::LineString(p) => {
                    p.lines().for_each(|sub_geom| {
                        self.idx.remove(&IndexItem {
                            id: id.clone(),
                            geom: Geometry::Line(sub_geom),
                        });
                    });
                },
                Geometry::MultiLineString(p) => {
                    p.into_iter().for_each(|line_string| {
                        line_string.lines().for_each(|sub_geom| {
                            self.idx.remove(&IndexItem {
                                id: id.clone(),
                                geom: Geometry::Line(sub_geom),
                            });
                        });
                    });
                },
                Geometry::MultiPolygon(p) => {
                    p.into_iter().for_each(|poly| {
                        self.idx.remove(&IndexItem {
                            id: id.clone(),
                            geom: Geometry::Polygon(poly),
                        });
                    });
                },
                _ => {
                    self.idx.remove(&IndexItem {
                        id: id.clone(),
                        geom,
                    });
                },
            }
        });
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
        true
    }

    fn index_linestring(&mut self, id: &str, p: LineString<f64>) {
        p.lines().for_each(|sub_geom| {
            self.idx.insert(IndexItem {
                id: id.to_owned(),
                geom: Geometry::Line(sub_geom),
            })
        });
    }

    pub fn nearest(&self, long: f64, lat: f64) -> Option<(String, String)> {
        self.idx
            .nearest_neighbor(&[long, lat])
            .map(|n| {
                (n.id.clone(), self.objects.get(&n.id).unwrap().clone())
            })
    }

    pub fn intersect(&self, long: f64, lat: f64) -> Option<(String, String)> {
        let nearest = self.idx.nearest_neighbor(&[long, lat]);
        if let Some(n) = nearest {
            if n.distance2(&[long, lat]) < 0.0000001 {
                return Some((n.id.clone(), self.objects.get(&n.id).unwrap().clone()));
            }
        }
        None
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