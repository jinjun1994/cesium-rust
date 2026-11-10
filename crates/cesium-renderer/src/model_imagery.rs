//! ModelImagery and ModelPrimitiveImagery matching CesiumJS `Scene/Model/ModelImagery.js` and `ModelPrimitiveImagery.js` (Apache-2.0).

use std::sync::{Arc, RwLock};

use cesium_math::{Color, DeveloperError};

use crate::imagery_layer::ImageryLayer;
use crate::model_primitive_imagery::ModelPrimitiveImagery;

/// A snapshot containing the values that affect the appearance of an ImageryLayer.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageryConfiguration {
    pub show: bool,
    pub alpha: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub hue: f32,
    pub saturation: f32,
    pub gamma: f32,
    pub color_to_alpha: Option<Color>,
}

impl ImageryConfiguration {
    pub fn from_layer(layer: &ImageryLayer) -> Self {
        Self {
            show: layer.show(),
            alpha: layer.alpha(),
            brightness: layer.brightness(),
            contrast: layer.contrast(),
            hue: layer.hue(),
            saturation: layer.saturation(),
            gamma: layer.gamma(),
            color_to_alpha: layer.color_to_alpha(),
        }
    }
}

/// A class managing the draping of imagery on a Model.
pub struct ModelImagery {
    primitive_count: usize,
    layers: Vec<Arc<RwLock<ImageryLayer>>>,
    model_primitive_imageries: Option<Vec<ModelPrimitiveImagery>>,
    imagery_configurations: Vec<ImageryConfiguration>,
    pub reset_draw_commands_called: bool,
    is_destroyed: bool,
}

impl ModelImagery {
    /// Creates a new ModelImagery instance.
    pub fn new(has_model: bool, primitive_count: usize) -> Result<Self, DeveloperError> {
        if !has_model {
            return Err(DeveloperError::new("model is required"));
        }
        Ok(Self {
            primitive_count,
            layers: Vec::new(),
            model_primitive_imageries: None,
            imagery_configurations: Vec::new(),
            reset_draw_commands_called: false,
            is_destroyed: false,
        })
    }

    pub fn set_layers(&mut self, layers: Vec<Arc<RwLock<ImageryLayer>>>) {
        self.layers = layers;
    }

    pub fn add_layer(&mut self, layer: Arc<RwLock<ImageryLayer>>) {
        self.layers.push(layer);
    }

    pub fn remove_all_layers(&mut self) {
        self.layers.clear();
    }

    pub fn has_imagery(&self) -> bool {
        !self.layers.is_empty()
    }

    pub fn all_imagery_layers_ready(&self) -> bool {
        if !self.has_imagery() {
            return true;
        }
        for layer_arc in &self.layers {
            if let Ok(layer) = layer_arc.read() {
                if !layer.is_ready() {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn all_model_primitive_imageries_ready(&self) -> bool {
        match &self.model_primitive_imageries {
            Some(imageries) => imageries.iter().all(|p| p.ready),
            None => false,
        }
    }

    pub fn ready(&self) -> bool {
        if !self.has_imagery() {
            return true;
        }
        if !self.all_imagery_layers_ready() {
            return false;
        }
        if !self.all_model_primitive_imageries_ready() {
            return false;
        }
        true
    }

    pub fn model_primitive_imageries(&self) -> Option<&[ModelPrimitiveImagery]> {
        self.model_primitive_imageries.as_deref()
    }

    pub fn update(&mut self) {
        if !self.has_imagery() {
            self.delete_model_primitive_imageries();
            return;
        }

        if !self.all_imagery_layers_ready() {
            return;
        }

        if self.model_primitive_imageries.is_none() {
            let mut list = Vec::with_capacity(self.primitive_count);
            for i in 0..self.primitive_count {
                list.push(ModelPrimitiveImagery::with_indices(0, i));
            }
            self.model_primitive_imageries = Some(list);
        }

        if let Some(imageries) = &mut self.model_primitive_imageries {
            for prim in imageries {
                prim.update();
            }
        }

        self.check_for_modified_imagery_configurations();
    }

    pub fn delete_model_primitive_imageries(&mut self) {
        if let Some(mut imageries) = self.model_primitive_imageries.take() {
            for prim in &mut imageries {
                prim.destroy();
            }
            self.reset_draw_commands();
        }
    }

    pub fn imagery_configurations_modified(&self) -> bool {
        if self.layers.len() != self.imagery_configurations.len() {
            return true;
        }
        for (i, layer_arc) in self.layers.iter().enumerate() {
            if let Ok(layer) = layer_arc.read() {
                let current = ImageryConfiguration::from_layer(&layer);
                if current != self.imagery_configurations[i] {
                    return true;
                }
            } else {
                return true;
            }
        }
        false
    }

    pub fn check_for_modified_imagery_configurations(&mut self) {
        if self.imagery_configurations_modified() {
            self.update_imagery_configurations();
            self.reset_draw_commands();
        }
    }

    pub fn update_imagery_configurations(&mut self) {
        self.imagery_configurations.clear();
        for layer_arc in &self.layers {
            if let Ok(layer) = layer_arc.read() {
                self.imagery_configurations
                    .push(ImageryConfiguration::from_layer(&layer));
            }
        }
    }

    pub fn reset_draw_commands(&mut self) {
        self.reset_draw_commands_called = true;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.delete_model_primitive_imageries();
        self.is_destroyed = true;
    }
}
