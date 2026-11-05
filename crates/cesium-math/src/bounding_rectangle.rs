//! Two-dimensional bounding rectangles adapted from CesiumJS (Apache-2.0).
use crate::{GeoRectangle, Intersect, MapProjection, MathError};
use glam::DVec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingRectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingRectangle {
    pub const PACKED_LENGTH: usize = 4;

    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points(points: &[DVec2]) -> Result<Self, MathError> {
        let Some((&first, rest)) = points.split_first() else {
            return Ok(Self::default());
        };
        if !first.is_finite() {
            return Err(MathError);
        }
        let (mut minimum, mut maximum) = (first, first);
        for &point in rest {
            if !point.is_finite() {
                return Err(MathError);
            }
            minimum = minimum.min(point);
            maximum = maximum.max(point);
        }
        Ok(Self::new(
            minimum.x,
            minimum.y,
            maximum.x - minimum.x,
            maximum.y - minimum.y,
        ))
    }

    /// Projects the southwest and northeast corners, matching Cesium's behavior
    /// even when the rectangle crosses the antimeridian.
    pub fn from_geographic<P: MapProjection + ?Sized>(
        rectangle: GeoRectangle,
        projection: &P,
    ) -> Option<Self> {
        let sw = projection.project(rectangle.southwest())?;
        let ne = projection.project(rectangle.northeast())?;
        Some(Self::new(sw.x, sw.y, ne.x - sw.x, ne.y - sw.y))
    }

    /// Computes a bounding rectangle from a geographic rectangle.
    /// If rectangle is None, returns default (all zeros).
    /// If projection is None, defaults to GeographicProjection.
    pub fn from_rectangle(
        rectangle: Option<&GeoRectangle>,
        projection: Option<&impl MapProjection>,
    ) -> Self {
        let Some(rect) = rectangle else {
            return Self::default();
        };
        let default_proj = crate::GeographicProjection::default();
        let proj: &dyn MapProjection = match projection {
            Some(p) => p,
            None => &default_proj,
        };
        Self::from_geographic(*rect, proj).unwrap_or_default()
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self::new(
            x,
            y,
            (self.x + self.width).max(other.x + other.width) - x,
            (self.y + self.height).max(other.y + other.height) - y,
        )
    }

    pub fn expand(mut self, point: DVec2) -> Self {
        let width = point.x - self.x;
        let height = point.y - self.y;
        if width > self.width {
            self.width = width;
        } else if width < 0.0 {
            self.width -= width;
            self.x = point.x;
        }
        if height > self.height {
            self.height = height;
        } else if height < 0.0 {
            self.height -= height;
            self.y = point.y;
        }
        self
    }

    /// Rectangles touching at an edge or corner intersect.
    pub fn intersects(self, other: Self) -> bool {
        !(self.x > other.x + other.width
            || self.x + self.width < other.x
            || self.y > other.y + other.height
            || self.y + self.height < other.y)
    }

    /// Determines whether two bounding rectangles intersect.
    pub fn intersect(&self, other: &Self) -> Intersect {
        if self.intersects(*other) {
            Intersect::INTERSECTING
        } else {
            Intersect::OUTSIDE
        }
    }

    /// Associated function for intersecting two bounding rectangles.
    pub fn intersect_rectangles(left: &Self, right: &Self) -> Intersect {
        left.intersect(right)
    }

    /// Compares two bounding rectangles for equality.
    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.x == o.x && self.y == o.y && self.width == o.width && self.height == o.height
            }
            None => false,
        }
    }

    pub fn pack(self, output: &mut [f64], offset: usize) -> Result<(), MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let target = output.get_mut(offset..end).ok_or(MathError)?;
        target.copy_from_slice(&[self.x, self.y, self.width, self.height]);
        Ok(())
    }

    pub fn unpack(input: &[f64], offset: usize) -> Result<Self, MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let values = input.get(offset..end).ok_or(MathError)?;
        Ok(Self::new(values[0], values[1], values[2], values[3]))
    }
}
