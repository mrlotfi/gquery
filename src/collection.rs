use std::collections::HashMap;
use geo_types::{GeometryCollection,Geometry, Point};
use spade::rtree::RTree;
use spade::{SpatialObject, BoundingRect};
use geo::prelude::*;
use geo::algorithm::bounding_rect::BoundingRect as br;
use geojson::{GeoJson, quick_collection};


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
        let mut collection: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
        collection.into_iter().for_each(|geom| {
            self.idx.insert(IndexItem {
                id: id.clone(),
                geom,
            })
        });
        self.objects.insert(id, geojson.to_string());
        self.nearest(2., 2.);
        true
    }

    pub fn nearest(&self, long: f64, lat: f64) -> (String, String) {
        let d = self.idx.nearest_neighbor(&[long, lat]).unwrap();
        return (d.id.clone(), self.objects.get(&d.id).unwrap().clone())
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use geo::{point, line_string, Geometry};
    #[test]
    fn test_add() {
        let mut c = Collection::new();
        c.idx.insert(IndexItem {
            id: String::from("123"),
            geom: Geometry::LineString(line_string![
                (x: 0., y: 1.),
                (x: 200., y: 1.),
            ]),
        });
        c.idx.insert(IndexItem {
            id: String::from("456"),
            geom: Geometry::LineString(line_string![
                (x: 49., y: -1.5),
                (x: 51., y: -1.5),
            ]),
        });
        let d = c.idx.nearest_neighbor(&[50.0, 0.0]);
        assert_eq!(1, 1);
    }
}