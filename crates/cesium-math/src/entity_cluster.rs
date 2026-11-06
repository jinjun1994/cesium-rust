//! Defines how screen space objects (billboards, points, labels) are clustered.
//! Direct port of CesiumJS `DataSources/EntityCluster.js`.

use crate::{
    billboard_collection::{Billboard, BillboardCollection},
    point_primitive_collection::{PointPrimitive, PointPrimitiveCollection},
    BoundingRectangle, Event,
};

/// Options for initializing an EntityCluster instance.
#[derive(Debug, Clone)]
pub struct EntityClusterOptions {
    pub enabled: bool,
    pub pixel_range: f64,
    pub minimum_cluster_size: usize,
    pub cluster_billboards: bool,
    pub cluster_labels: bool,
    pub cluster_points: bool,
    pub show: bool,
}

impl Default for EntityClusterOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            pixel_range: 80.0,
            minimum_cluster_size: 2,
            cluster_billboards: true,
            cluster_labels: true,
            cluster_points: true,
            show: true,
        }
    }
}

/// Defines how screen space objects (billboards, points, labels) are clustered.
#[derive(Clone)]
pub struct EntityCluster {
    pub enabled: bool,
    pub pixel_range: f64,
    pub minimum_cluster_size: usize,
    pub cluster_billboards: bool,
    pub cluster_labels: bool,
    pub cluster_points: bool,
    pub show: bool,
    pub cluster_event: Event<(Vec<String>, usize)>, // (clustered_ids, count)
    pub billboard_collection: Option<BillboardCollection>,
    pub point_collection: Option<PointPrimitiveCollection>,
}

impl Default for EntityCluster {
    fn default() -> Self {
        Self::new(EntityClusterOptions::default())
    }
}

impl EntityCluster {
    /// Creates a new EntityCluster with the given options.
    pub fn new(options: EntityClusterOptions) -> Self {
        Self {
            enabled: options.enabled,
            pixel_range: options.pixel_range,
            minimum_cluster_size: options.minimum_cluster_size,
            cluster_billboards: options.cluster_billboards,
            cluster_labels: options.cluster_labels,
            cluster_points: options.cluster_points,
            show: options.show,
            cluster_event: Event::new(),
            billboard_collection: None,
            point_collection: None,
        }
    }

    /// Gets or creates the billboard collection.
    pub fn get_or_create_billboard_collection(&mut self) -> &mut BillboardCollection {
        if self.billboard_collection.is_none() {
            self.billboard_collection = Some(BillboardCollection::new());
        }
        self.billboard_collection.as_mut().unwrap()
    }

    /// Gets or allocates a billboard associated with an entity. Reuses unused billboards.
    pub fn get_billboard(&mut self, entity_id: &str) -> &mut Billboard {
        let collection = self.get_or_create_billboard_collection();
        if let Some(pos) = collection
            .billboards
            .iter()
            .position(|b| b.id.as_deref() == Some(entity_id))
        {
            return &mut collection.billboards[pos];
        }
        if let Some(pos) = collection.billboards.iter().position(|b| b.id.is_none()) {
            let bb = &mut collection.billboards[pos];
            bb.id = Some(entity_id.to_string());
            return bb;
        }
        let bb = Billboard {
            id: Some(entity_id.to_string()),
            ..Default::default()
        };
        collection.billboards.push(bb);
        collection.billboards.last_mut().unwrap()
    }

    /// Removes an entity's billboard, hiding it and marking it for reuse.
    pub fn remove_billboard(&mut self, entity_id: &str) {
        if let Some(col) = &mut self.billboard_collection {
            if let Some(bb) = col
                .billboards
                .iter_mut()
                .find(|b| b.id.as_deref() == Some(entity_id))
            {
                bb.show = false;
                bb.id = None;
            }
        }
    }

    /// Gets or creates the point primitive collection.
    pub fn get_or_create_point_collection(&mut self) -> &mut PointPrimitiveCollection {
        if self.point_collection.is_none() {
            self.point_collection = Some(PointPrimitiveCollection::new());
        }
        self.point_collection.as_mut().unwrap()
    }

    /// Gets or allocates a point primitive associated with an entity. Reuses unused primitives.
    pub fn get_point(&mut self, entity_id: &str) -> &mut PointPrimitive {
        let collection = self.get_or_create_point_collection();
        if let Some(pos) = collection
            .point_primitives
            .iter()
            .position(|p| p.id.as_deref() == Some(entity_id))
        {
            return &mut collection.point_primitives[pos];
        }
        if let Some(pos) = collection.point_primitives.iter().position(|p| p.id.is_none()) {
            let p = &mut collection.point_primitives[pos];
            p.id = Some(entity_id.to_string());
            return p;
        }
        let p = PointPrimitive {
            id: Some(entity_id.to_string()),
            ..Default::default()
        };
        collection.point_primitives.push(p);
        collection.point_primitives.last_mut().unwrap()
    }

    /// Removes an entity's point primitive, hiding it and marking it for reuse.
    pub fn remove_point(&mut self, entity_id: &str) {
        if let Some(col) = &mut self.point_collection {
            if let Some(p) = col
                .point_primitives
                .iter_mut()
                .find(|p| p.id.as_deref() == Some(entity_id))
            {
                p.show = false;
                p.id = None;
            }
        }
    }

    /// Expands a 2D bounding rectangle by the pixel range.
    pub fn expand_bounding_box(bbox: &mut BoundingRectangle, pixel_range: f64) {
        bbox.x -= pixel_range;
        bbox.y -= pixel_range;
        bbox.width += pixel_range * 2.0;
        bbox.height += pixel_range * 2.0;
    }
}
