//! SelectionIndicatorViewModel matching CesiumJS `Widgets/SelectionIndicator/SelectionIndicatorViewModel.js` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian3, DeveloperError};
use std::cell::RefCell;
use std::rc::Rc;

pub const OFF_SCREEN_COORDINATE: &str = "-1000px";

/// Simulated Scene state for SelectionIndicator.
#[derive(Clone, Debug, Default)]
pub struct SelectionIndicatorScene {
    pub is_destroyed: bool,
}

impl SelectionIndicatorScene {
    /// Creates a new `SelectionIndicatorScene`.
    pub fn new() -> Self {
        Self {
            is_destroyed: false,
        }
    }

    /// Destroys scene resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// Simulated DOM element for the selection indicator visual element.
#[derive(Clone, Debug)]
pub struct SelectionIndicatorElement {
    pub client_width: f64,
    pub client_height: f64,
}

impl Default for SelectionIndicatorElement {
    fn default() -> Self {
        Self {
            client_width: 20.0,
            client_height: 20.0,
        }
    }
}

impl SelectionIndicatorElement {
    /// Creates a new `SelectionIndicatorElement`.
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            client_width: width,
            client_height: height,
        }
    }
}

/// Simulated container DOM element for SelectionIndicator.
#[derive(Clone, Debug)]
pub struct SelectionIndicatorContainer {
    pub id: String,
    pub parent_node_client_width: f64,
    pub parent_node_client_height: f64,
}

impl Default for SelectionIndicatorContainer {
    fn default() -> Self {
        Self {
            id: String::new(),
            parent_node_client_width: 800.0,
            parent_node_client_height: 600.0,
        }
    }
}

impl SelectionIndicatorContainer {
    /// Creates a new container with an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            parent_node_client_width: 800.0,
            parent_node_client_height: 600.0,
        }
    }
}

/// Type of closure converting world coordinates to window screen space coordinates.
pub type ComputeScreenSpacePositionFn =
    Box<dyn Fn(&Cartesian3) -> Option<Cartesian2>>;

/// View model for [`SelectionIndicator`].
pub struct SelectionIndicatorViewModel {
    scene: Rc<RefCell<SelectionIndicatorScene>>,
    selection_indicator_element: Rc<RefCell<SelectionIndicatorElement>>,
    container: Rc<RefCell<SelectionIndicatorContainer>>,
    pub position: Option<Cartesian3>,
    pub show_selection: bool,
    scale: f64,
    screen_position_x: String,
    screen_position_y: String,
    compute_screen_space_position: ComputeScreenSpacePositionFn,
}

impl SelectionIndicatorViewModel {
    /// Creates a new `SelectionIndicatorViewModel`.
    pub fn new(
        scene: Option<Rc<RefCell<SelectionIndicatorScene>>>,
        selection_indicator_element: Option<Rc<RefCell<SelectionIndicatorElement>>>,
        container: Option<Rc<RefCell<SelectionIndicatorContainer>>>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;
        let selection_indicator_element = selection_indicator_element
            .ok_or_else(|| DeveloperError::new("selectionIndicatorElement is required."))?;
        let container =
            container.ok_or_else(|| DeveloperError::new("container is required."))?;

        let default_compute: ComputeScreenSpacePositionFn =
            Box::new(|pos| Some(Cartesian2::new(pos.x, pos.y)));

        Ok(Self {
            scene,
            selection_indicator_element,
            container,
            position: None,
            show_selection: false,
            scale: 1.0,
            screen_position_x: OFF_SCREEN_COORDINATE.to_string(),
            screen_position_y: OFF_SCREEN_COORDINATE.to_string(),
            compute_screen_space_position: default_compute,
        })
    }

    /// Gets the scene.
    pub fn scene(&self) -> Rc<RefCell<SelectionIndicatorScene>> {
        Rc::clone(&self.scene)
    }

    /// Gets the selection indicator DOM element.
    pub fn selection_indicator_element(&self) -> Rc<RefCell<SelectionIndicatorElement>> {
        Rc::clone(&self.selection_indicator_element)
    }

    /// Gets the container DOM element.
    pub fn container(&self) -> Rc<RefCell<SelectionIndicatorContainer>> {
        Rc::clone(&self.container)
    }

    /// Gets the current transform scale.
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Gets whether the selection indicator is visible.
    pub fn is_visible(&self) -> bool {
        self.show_selection && self.position.is_some()
    }

    /// Gets screen position X (e.g. "-1000px" or "-9px").
    pub fn screen_position_x(&self) -> &str {
        &self.screen_position_x
    }

    /// Gets screen position Y (e.g. "-1000px" or "-8px").
    pub fn screen_position_y(&self) -> &str {
        &self.screen_position_y
    }

    /// Sets a custom screen-space position computation function.
    pub fn set_compute_screen_space_position<F>(&mut self, compute: F)
    where
        F: Fn(&Cartesian3) -> Option<Cartesian2> + 'static,
    {
        self.compute_screen_space_position = Box::new(compute);
    }

    /// Updates screen coordinates based on current position and container geometry.
    pub fn update(&mut self) {
        if self.show_selection {
            if let Some(pos) = &self.position {
                if let Some(screen_pos) = (self.compute_screen_space_position)(pos) {
                    let container_w = self.container.borrow().parent_node_client_width;
                    let container_h = self.container.borrow().parent_node_client_height;
                    let indicator_size =
                        self.selection_indicator_element.borrow().client_width;
                    let half_size = indicator_size * 0.5;

                    let clamped_x = screen_pos
                        .x
                        .max(-indicator_size)
                        .min(container_w + indicator_size)
                        - half_size;
                    let clamped_y = screen_pos
                        .y
                        .max(-indicator_size)
                        .min(container_h + indicator_size)
                        - half_size;

                    self.screen_position_x =
                        format!("{}px", (clamped_x + 0.25).floor() as i64);
                    self.screen_position_y =
                        format!("{}px", (clamped_y + 0.25).floor() as i64);
                    return;
                }
            }
        }
        self.screen_position_x = OFF_SCREEN_COORDINATE.to_string();
        self.screen_position_y = OFF_SCREEN_COORDINATE.to_string();
    }

    /// Animates appearance of the selection indicator.
    pub fn animate_appear(&mut self) {
        self.scale = 1.0;
    }

    /// Animates departure of the selection indicator.
    pub fn animate_depart(&mut self) {
        self.scale = 1.5;
    }
}
