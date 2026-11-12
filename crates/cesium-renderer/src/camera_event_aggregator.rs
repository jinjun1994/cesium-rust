//! CameraEventAggregator matching CesiumJS `Scene/CameraEventAggregator.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Cartesian2, KeyboardEventModifier};

/// Types of camera input events aggregated by [`CameraEventAggregator`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CameraEventType {
    LeftDrag,
    RightDrag,
    MiddleDrag,
    Wheel,
    Pinch,
}

/// Start and end screen-space positions of an aggregated camera movement.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CameraMovement {
    pub start_position: Cartesian2,
    pub end_position: Cartesian2,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct EventKey {
    event_type: CameraEventType,
    modifier: Option<KeyboardEventModifier>,
}

#[derive(Clone, Debug, Default)]
struct EventState {
    is_down: bool,
    is_moving: bool,
    press_time: Option<f64>,
    release_time: Option<f64>,
    start_mouse_position: Option<Cartesian2>,
    movement: Option<CameraMovement>,
    last_movement: Option<CameraMovement>,
}

/// Aggregates camera input events (mouse drags, wheel, pinch) between frames.
#[derive(Clone, Debug, Default)]
pub struct CameraEventAggregator {
    states: HashMap<EventKey, EventState>,
    any_down_count: usize,
    destroyed: bool,
}

impl CameraEventAggregator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn destroy(&mut self) {
        self.states.clear();
        self.any_down_count = 0;
        self.destroyed = true;
    }

    pub fn any_button_down(&self) -> bool {
        self.any_down_count > 0
    }

    pub fn is_button_down(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> bool {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .is_some_and(|s| s.is_down)
    }

    pub fn is_moving(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> bool {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .is_some_and(|s| s.is_moving)
    }

    pub fn get_movement(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> Option<CameraMovement> {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .and_then(|s| s.movement)
    }

    pub fn get_last_movement(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> Option<CameraMovement> {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .and_then(|s| s.last_movement)
    }

    pub fn get_button_press_time(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> Option<f64> {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .and_then(|s| s.press_time)
    }

    pub fn get_button_release_time(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> Option<f64> {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .and_then(|s| s.release_time)
    }

    pub fn get_start_mouse_position(
        &self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
    ) -> Option<Cartesian2> {
        self.states
            .get(&EventKey {
                event_type,
                modifier,
            })
            .and_then(|s| s.start_mouse_position)
    }

    pub fn on_mouse_down(
        &mut self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
        position: Cartesian2,
        timestamp: f64,
    ) {
        let key = EventKey {
            event_type,
            modifier,
        };
        let state = self.states.entry(key).or_default();
        if !state.is_down {
            state.is_down = true;
            self.any_down_count += 1;
        }
        state.press_time = Some(timestamp);
        state.start_mouse_position = Some(position);
    }

    pub fn on_mouse_up(
        &mut self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
        _position: Cartesian2,
        timestamp: f64,
    ) {
        // Any button up cancels any_down_count for that key and clears any_button_down as in CesiumJS
        let key = EventKey {
            event_type,
            modifier,
        };
        if let Some(state) = self.states.get_mut(&key) {
            if state.is_down {
                state.is_down = false;
            }
            state.release_time = Some(timestamp);
        }
        // CesiumJS cancels anyButtonDown when any button is released
        self.any_down_count = 0;
        for s in self.states.values() {
            if s.is_down {
                // Only count buttons still down on other keys if not cancelled
            }
        }
    }

    pub fn on_mouse_move(
        &mut self,
        event_type: CameraEventType,
        modifier: Option<KeyboardEventModifier>,
        start_position: Cartesian2,
        end_position: Cartesian2,
    ) {
        let key = EventKey {
            event_type,
            modifier,
        };
        let state = self.states.entry(key).or_default();
        state.is_moving = true;
        match &mut state.movement {
            Some(mov) if mov.valid => {
                mov.end_position = end_position;
            }
            _ => {
                state.movement = Some(CameraMovement {
                    start_position,
                    end_position,
                    valid: true,
                });
            }
        }
    }

    pub fn on_wheel(&mut self, modifier: Option<KeyboardEventModifier>, delta: f64) {
        let key = EventKey {
            event_type: CameraEventType::Wheel,
            modifier,
        };
        let state = self.states.entry(key).or_default();
        state.is_moving = true;
        match &mut state.movement {
            Some(mov) if mov.valid => {
                mov.end_position.y += delta;
            }
            _ => {
                state.movement = Some(CameraMovement {
                    start_position: Cartesian2::ZERO,
                    end_position: Cartesian2::new(0.0, delta),
                    valid: true,
                });
            }
        }
    }

    /// Resets per-frame movement state while preserving button-down state and `last_movement`.
    pub fn reset(&mut self) {
        for state in self.states.values_mut() {
            if let Some(mov) = state.movement.take() {
                state.last_movement = Some(mov);
            }
            state.is_moving = false;
        }
    }
}
