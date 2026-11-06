//! TerrainOffsetProperty matching CesiumJS `DataSources/TerrainOffsetProperty.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian3, DeveloperError, DynProperty, Event, HeightReference, JulianDate,
};

/// A property that calculates height offsets relative to terrain.
pub struct TerrainOffsetProperty {
    position_property: Rc<dyn DynProperty<Cartesian3>>,
    height_reference_raw: Option<Rc<dyn DynProperty<f64>>>,
    extruded_height_reference_raw: Option<Rc<dyn DynProperty<f64>>>,
    definition_changed: Event<()>,
    terrain_height: f64,
    destroyed: bool,
}

impl TerrainOffsetProperty {
    /// Creates a new TerrainOffsetProperty instance.
    pub fn new(
        scene: Option<()>,
        position_property: Option<Rc<dyn DynProperty<Cartesian3>>>,
        height_reference: Option<Rc<dyn DynProperty<f64>>>,
        extruded_height_reference: Option<Rc<dyn DynProperty<f64>>>,
    ) -> Result<Self, DeveloperError> {
        if scene.is_none() {
            return Err(DeveloperError::new("scene is required."));
        }
        let pos_prop = position_property
            .ok_or_else(|| DeveloperError::new("positionProperty is required."))?;

        Ok(Self {
            position_property: pos_prop,
            height_reference_raw: height_reference,
            extruded_height_reference_raw: extruded_height_reference,
            definition_changed: Event::new(),
            terrain_height: 0.0,
            destroyed: false,
        })
    }

    /// Gets a value indicating if this property is constant. Always false for terrain offsets.
    pub fn is_constant(&self) -> bool {
        false
    }

    /// Gets the event that is raised whenever the definition of this property changes.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Gets the offset at the provided time.
    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        let height_ref_val = self
            .height_reference_raw
            .as_ref()
            .and_then(|p| p.get_value(time))
            .unwrap_or(0.0);
        let extruded_ref_val = self
            .extruded_height_reference_raw
            .as_ref()
            .and_then(|p| p.get_value(time))
            .unwrap_or(0.0);

        let height_ref = height_reference_from_val(height_ref_val);
        let extruded_ref = height_reference_from_val(extruded_ref_val);

        if height_ref == HeightReference::None && !is_height_reference_relative(extruded_ref) {
            return Some(Cartesian3::ZERO);
        }

        let pos = self.position_property.get_value(time)?;
        if pos == Cartesian3::ZERO {
            return Some(Cartesian3::ZERO);
        }

        // Return offset along ellipsoid surface normal
        let normal = pos.normalize();
        Some(normal * self.terrain_height)
    }

    /// Destroys this property.
    pub fn destroy(&mut self) {
        self.destroyed = true;
    }

    /// Gets whether this property has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }
}

fn height_reference_from_val(v: f64) -> HeightReference {
    let int_val = v.round() as i32;
    match int_val {
        1 => HeightReference::ClampToTerrain,
        2 => HeightReference::RelativeToTerrain,
        3 => HeightReference::ClampTo3DTiles,
        4 => HeightReference::RelativeTo3DTiles,
        5 => HeightReference::ClampToGround,
        6 => HeightReference::RelativeToGround,
        _ => HeightReference::None,
    }
}

fn is_height_reference_relative(hr: HeightReference) -> bool {
    matches!(
        hr,
        HeightReference::RelativeToTerrain
            | HeightReference::RelativeTo3DTiles
            | HeightReference::RelativeToGround
    )
}
