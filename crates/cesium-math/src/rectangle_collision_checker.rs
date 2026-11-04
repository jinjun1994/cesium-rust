use crate::GeoRectangle;

#[derive(Debug, Clone, PartialEq)]
struct RectangleEntry {
    id: String,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
}

impl RectangleEntry {
    fn intersects(&self, other_west: f64, other_south: f64, other_east: f64, other_north: f64) -> bool {
        self.west <= other_east
            && self.east >= other_west
            && self.south <= other_north
            && self.north >= other_south
    }
}

/// Checks for 2D rectangle collisions in geographic coordinates.
#[derive(Debug, Clone, Default)]
pub struct RectangleCollisionChecker {
    entries: Vec<RectangleEntry>,
}

impl RectangleCollisionChecker {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Insert a rectangle with an identifier into the collection.
    pub fn insert(&mut self, id: impl Into<String>, rectangle: &GeoRectangle) {
        self.entries.push(RectangleEntry {
            id: id.into(),
            west: rectangle.west,
            south: rectangle.south,
            east: rectangle.east,
            north: rectangle.north,
        });
    }

    /// Remove a rectangle from the collection matching id and bounds.
    pub fn remove(&mut self, id: &str, _rectangle: &GeoRectangle) {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.entries.remove(pos);
        }
    }

    /// Checks if a given rectangle collides with any of the rectangles in the collection.
    pub fn collides(&self, rectangle: &GeoRectangle) -> bool {
        self.entries.iter().any(|e| {
            e.intersects(rectangle.west, rectangle.south, rectangle.east, rectangle.north)
        })
    }
}
