//! TileImagery matching CesiumJS `Scene/TileImagery.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::Cartesian4;

use crate::imagery_pipeline_stage::ImageryState;

/// Representation of an imagery node in the tile hierarchy.
#[derive(Clone, Debug)]
pub struct ImageryNode {
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub state: ImageryState,
    pub texture: Option<String>,
    pub texture_web_mercator: Option<String>,
    pub parent: Option<Rc<RefCell<ImageryNode>>>,
    pub process_state_machine_calls: usize,
    pub reference_count: usize,
}

impl ImageryNode {
    /// Creates a new `ImageryNode`.
    pub fn new(level: u32, x: u32, y: u32, state: ImageryState) -> Self {
        Self {
            level,
            x,
            y,
            state,
            texture: None,
            texture_web_mercator: None,
            parent: None,
            process_state_machine_calls: 0,
            reference_count: 0,
        }
    }

    pub fn add_reference(&mut self) {
        self.reference_count += 1;
    }

    pub fn release_reference(&mut self) {
        if self.reference_count > 0 {
            self.reference_count -= 1;
        }
    }

    pub fn process_state_machine(&mut self) {
        self.process_state_machine_calls += 1;
    }
}

/// The association between a terrain tile and an imagery tile.
#[derive(Clone, Debug)]
pub struct TileImagery {
    pub ready_imagery: Option<Rc<RefCell<ImageryNode>>>,
    pub loading_imagery: Option<Rc<RefCell<ImageryNode>>>,
    pub texture_coordinate_rectangle: Option<Cartesian4>,
    pub texture_translation_and_scale: Option<Cartesian4>,
    pub use_web_mercator_t: bool,
}

impl TileImagery {
    /// Creates a new `TileImagery`.
    pub fn new(
        loading_imagery: Option<Rc<RefCell<ImageryNode>>>,
        texture_coordinate_rectangle: Option<Cartesian4>,
        use_web_mercator_t: bool,
    ) -> Self {
        if let Some(loading) = &loading_imagery {
            loading.borrow_mut().add_reference();
        }

        Self {
            ready_imagery: None,
            loading_imagery,
            texture_coordinate_rectangle,
            texture_translation_and_scale: None,
            use_web_mercator_t,
        }
    }

    /// Frees the resources held by this instance.
    pub fn free_resources(&mut self) {
        if let Some(ready) = &self.ready_imagery {
            ready.borrow_mut().release_reference();
        }
        if let Some(loading) = &self.loading_imagery {
            loading.borrow_mut().release_reference();
        }
    }

    /// Processes the load state machine for this instance.
    pub fn process_state_machine(&mut self) -> bool {
        let loading_imagery = match &self.loading_imagery {
            Some(img) => Rc::clone(img),
            None => return true,
        };

        loading_imagery.borrow_mut().process_state_machine();

        let state = loading_imagery.borrow().state;
        if state == ImageryState::Ready {
            if let Some(ready) = &self.ready_imagery {
                ready.borrow_mut().release_reference();
            }
            self.ready_imagery = Some(Rc::clone(&loading_imagery));
            self.loading_imagery = None;
            self.texture_translation_and_scale = Some(Cartesian4::new(0.0, 0.0, 1.0, 1.0));
            return true;
        }

        // Find some ancestor imagery we can use while this imagery is still loading.
        let mut ancestor = loading_imagery.borrow().parent.clone();
        let mut closest_ancestor_that_needs_loading = None;

        while let Some(current) = ancestor {
            let is_ready = current.borrow().state == ImageryState::Ready;
            let needs_reproject = !self.use_web_mercator_t && current.borrow().texture.is_none();

            if is_ready && !needs_reproject {
                ancestor = Some(current);
                break;
            }

            let cur_state = current.borrow().state;
            if cur_state != ImageryState::Failed
                && cur_state != ImageryState::Invalid
                && closest_ancestor_that_needs_loading.is_none()
            {
                closest_ancestor_that_needs_loading = Some(Rc::clone(&current));
            }

            ancestor = current.borrow().parent.clone();
        }

        let is_different = match (&self.ready_imagery, &ancestor) {
            (Some(a), Some(b)) => !Rc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        if is_different {
            if let Some(ready) = &self.ready_imagery {
                ready.borrow_mut().release_reference();
            }

            self.ready_imagery = ancestor.clone();

            if let Some(ready) = &self.ready_imagery {
                ready.borrow_mut().add_reference();
                self.texture_translation_and_scale = Some(Cartesian4::new(0.0, 0.0, 1.0, 1.0));
            }
        }

        let loading_state = loading_imagery.borrow().state;
        if loading_state == ImageryState::Failed || loading_state == ImageryState::Invalid {
            if let Some(closest) = closest_ancestor_that_needs_loading {
                closest.borrow_mut().process_state_machine();
                return false;
            }
            return true;
        }

        false
    }
}
