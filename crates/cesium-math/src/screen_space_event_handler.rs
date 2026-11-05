use std::collections::HashMap;
use std::sync::Arc;

use crate::{Cartesian2, DeveloperError};

/// Mouse and touch event types for screen space input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ScreenSpaceEventType {
    LeftDown = 0,
    LeftUp = 1,
    LeftClick = 2,
    LeftDoubleClick = 3,
    RightDown = 5,
    RightUp = 6,
    RightClick = 7,
    MiddleDown = 10,
    MiddleUp = 11,
    MiddleClick = 12,
    MouseMove = 15,
    Wheel = 16,
    PinchStart = 17,
    PinchEnd = 18,
    PinchMove = 19,
}

impl ScreenSpaceEventType {
    pub const LEFT_DOWN: Self = Self::LeftDown;
    pub const LEFT_UP: Self = Self::LeftUp;
    pub const LEFT_CLICK: Self = Self::LeftClick;
    pub const LEFT_DOUBLE_CLICK: Self = Self::LeftDoubleClick;
    pub const RIGHT_DOWN: Self = Self::RightDown;
    pub const RIGHT_UP: Self = Self::RightUp;
    pub const RIGHT_CLICK: Self = Self::RightClick;
    pub const MIDDLE_DOWN: Self = Self::MiddleDown;
    pub const MIDDLE_UP: Self = Self::MiddleUp;
    pub const MIDDLE_CLICK: Self = Self::MiddleClick;
    pub const MOUSE_MOVE: Self = Self::MouseMove;
    pub const WHEEL: Self = Self::Wheel;
    pub const PINCH_START: Self = Self::PinchStart;
    pub const PINCH_END: Self = Self::PinchEnd;
    pub const PINCH_MOVE: Self = Self::PinchMove;
}

/// Keyboard modifiers for screen space events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum KeyboardEventModifier {
    Shift = 0,
    Ctrl = 1,
    Alt = 2,
}

impl KeyboardEventModifier {
    pub const SHIFT: Self = Self::Shift;
    pub const CTRL: Self = Self::Ctrl;
    pub const ALT: Self = Self::Alt;
}

/// An Event that occurs at a single position on screen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PositionedEvent {
    pub position: Cartesian2,
}

/// An Event that starts at one position and ends at another.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionEvent {
    pub start_position: Cartesian2,
    pub end_position: Cartesian2,
}

/// An Event that occurs for pinch / two-finger gestures.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PinchMovementEvent {
    pub distance: MotionEvent,
    pub angle_and_height: MotionEvent,
}

/// Generic input action handler callback.
#[derive(Clone)]
pub enum InputAction {
    Positioned(Arc<dyn Fn(&PositionedEvent) + Send + Sync>),
    Motion(Arc<dyn Fn(&MotionEvent) + Send + Sync>),
    Wheel(Arc<dyn Fn(f64) + Send + Sync>),
    PinchMovement(Arc<dyn Fn(&PinchMovementEvent) + Send + Sync>),
    Empty(Arc<dyn Fn() + Send + Sync>),
}

/// Computes the unique key for input action storage given event type and modifiers.
pub fn get_input_event_key(
    event_type: ScreenSpaceEventType,
    modifiers: Option<&[KeyboardEventModifier]>,
) -> String {
    let type_id = event_type as u8;
    if let Some(mods) = modifiers {
        if mods.is_empty() {
            type_id.to_string()
        } else {
            let mut sorted_mods: Vec<u8> = mods.iter().map(|&m| m as u8).collect();
            sorted_mods.sort_unstable();
            let mod_strs: Vec<String> = sorted_mods.iter().map(|m| m.to_string()).collect();
            format!("{}+{}", type_id, mod_strs.join("+"))
        }
    } else {
        type_id.to_string()
    }
}

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Handles user input events and executes registered actions.
pub struct ScreenSpaceEventHandler {
    input_events: HashMap<String, InputAction>,
    click_pixel_tolerance: f64,
    destroyed: bool,

    // Internal gesture state
    primary_start_position: Cartesian2,
    primary_previous_position: Cartesian2,
    button_down: [bool; 3], // Left, Middle, Right
    is_pinching: bool,
    pinch_previous_pos1: Cartesian2,
    pinch_previous_pos2: Cartesian2,
}

impl Default for ScreenSpaceEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenSpaceEventHandler {
    /// Creates a new ScreenSpaceEventHandler.
    pub fn new() -> Self {
        Self {
            input_events: HashMap::new(),
            click_pixel_tolerance: 5.0,
            destroyed: false,
            primary_start_position: Cartesian2::ZERO,
            primary_previous_position: Cartesian2::ZERO,
            button_down: [false; 3],
            is_pinching: false,
            pinch_previous_pos1: Cartesian2::ZERO,
            pinch_previous_pos2: Cartesian2::ZERO,
        }
    }

    /// Sets the click pixel tolerance in pixels.
    pub fn set_click_pixel_tolerance(&mut self, tolerance: f64) {
        self.click_pixel_tolerance = tolerance;
    }

    /// Returns the click pixel tolerance.
    pub fn click_pixel_tolerance(&self) -> f64 {
        self.click_pixel_tolerance
    }

    /// Set an action to be executed on an input event.
    pub fn set_input_action(
        &mut self,
        action: InputAction,
        event_type: ScreenSpaceEventType,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) {
        let key = get_input_event_key(event_type, modifiers);
        self.input_events.insert(key, action);
    }

    /// Returns the action to be executed on an input event if registered.
    pub fn get_input_action(
        &self,
        event_type: ScreenSpaceEventType,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Option<&InputAction> {
        let key = get_input_event_key(event_type, modifiers);
        self.input_events.get(&key)
    }

    /// Removes the action associated with the specified event and modifiers.
    pub fn remove_input_action(
        &mut self,
        event_type: ScreenSpaceEventType,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) {
        let key = get_input_event_key(event_type, modifiers);
        self.input_events.remove(&key);
    }

    /// Dispatches a mouse button down event.
    pub fn handle_mouse_down(
        &mut self,
        button: MouseButton,
        position: Cartesian2,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;

        let btn_idx = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
        };
        self.button_down[btn_idx] = true;
        self.primary_start_position = position;
        self.primary_previous_position = position;

        let event_type = match button {
            MouseButton::Left => ScreenSpaceEventType::LeftDown,
            MouseButton::Middle => ScreenSpaceEventType::MiddleDown,
            MouseButton::Right => ScreenSpaceEventType::RightDown,
        };

        if let Some(InputAction::Positioned(cb)) = self.get_input_action(event_type, modifiers) {
            let ev = PositionedEvent { position };
            cb(&ev);
        }

        Ok(())
    }

    /// Dispatches a mouse button up event, automatically firing click if within pixel tolerance.
    pub fn handle_mouse_up(
        &mut self,
        button: MouseButton,
        position: Cartesian2,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;

        let btn_idx = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
        };
        self.button_down[btn_idx] = false;

        let (up_type, click_type) = match button {
            MouseButton::Left => (ScreenSpaceEventType::LeftUp, ScreenSpaceEventType::LeftClick),
            MouseButton::Middle => (
                ScreenSpaceEventType::MiddleUp,
                ScreenSpaceEventType::MiddleClick,
            ),
            MouseButton::Right => (
                ScreenSpaceEventType::RightUp,
                ScreenSpaceEventType::RightClick,
            ),
        };

        if let Some(InputAction::Positioned(cb)) = self.get_input_action(up_type, modifiers) {
            let ev = PositionedEvent { position };
            cb(&ev);
        }

        let dist_sq = (position.x - self.primary_start_position.x).powi(2)
            + (position.y - self.primary_start_position.y).powi(2);
        if dist_sq <= self.click_pixel_tolerance * self.click_pixel_tolerance {
            if let Some(InputAction::Positioned(cb)) = self.get_input_action(click_type, modifiers)
            {
                let ev = PositionedEvent { position };
                cb(&ev);
            }
        }

        Ok(())
    }

    /// Dispatches a mouse move event.
    pub fn handle_mouse_move(
        &mut self,
        position: Cartesian2,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;

        let prev = self.primary_previous_position;
        self.primary_previous_position = position;

        if let Some(InputAction::Motion(cb)) =
            self.get_input_action(ScreenSpaceEventType::MouseMove, modifiers)
        {
            let ev = MotionEvent {
                start_position: prev,
                end_position: position,
            };
            cb(&ev);
        }

        Ok(())
    }

    /// Dispatches a mouse wheel event.
    pub fn handle_wheel(
        &self,
        delta: f64,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;

        if let Some(InputAction::Wheel(cb)) =
            self.get_input_action(ScreenSpaceEventType::Wheel, modifiers)
        {
            cb(delta);
        }

        Ok(())
    }

    /// Dispatches a pinch start event.
    pub fn handle_pinch_start(
        &mut self,
        pos1: Cartesian2,
        pos2: Cartesian2,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        self.is_pinching = true;
        self.pinch_previous_pos1 = pos1;
        self.pinch_previous_pos2 = pos2;

        if let Some(InputAction::Empty(cb)) =
            self.get_input_action(ScreenSpaceEventType::PinchStart, modifiers)
        {
            cb();
        }

        Ok(())
    }

    /// Dispatches a pinch move event.
    pub fn handle_pinch_move(
        &mut self,
        pos1: Cartesian2,
        pos2: Cartesian2,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        if !self.is_pinching {
            return Ok(());
        }

        let prev_pos1 = self.pinch_previous_pos1;
        let prev_pos2 = self.pinch_previous_pos2;

        let dx = pos2.x - pos1.x;
        let dy = pos2.y - pos1.y;
        let dist = (dx * dx + dy * dy).sqrt() * 0.25;

        let prev_dx = prev_pos2.x - prev_pos1.x;
        let prev_dy = prev_pos2.y - prev_pos1.y;
        let prev_dist = (prev_dx * prev_dx + prev_dy * prev_dy).sqrt() * 0.25;

        let cy = (pos2.y + pos1.y) * 0.125;
        let prev_cy = (prev_pos2.y + prev_pos1.y) * 0.125;

        let angle = dy.atan2(dx);
        let prev_angle = prev_dy.atan2(prev_dx);

        self.pinch_previous_pos1 = pos1;
        self.pinch_previous_pos2 = pos2;

        if let Some(InputAction::PinchMovement(cb)) =
            self.get_input_action(ScreenSpaceEventType::PinchMove, modifiers)
        {
            let ev = PinchMovementEvent {
                distance: MotionEvent {
                    start_position: Cartesian2::new(0.0, prev_dist),
                    end_position: Cartesian2::new(0.0, dist),
                },
                angle_and_height: MotionEvent {
                    start_position: Cartesian2::new(prev_angle, prev_cy),
                    end_position: Cartesian2::new(angle, cy),
                },
            };
            cb(&ev);
        }

        Ok(())
    }

    /// Dispatches a pinch end event.
    pub fn handle_pinch_end(
        &mut self,
        modifiers: Option<&[KeyboardEventModifier]>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        if self.is_pinching {
            self.is_pinching = false;
            if let Some(InputAction::Empty(cb)) =
                self.get_input_action(ScreenSpaceEventType::PinchEnd, modifiers)
            {
                cb();
            }
        }
        Ok(())
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Destroys this handler and clears all registered actions.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        self.input_events.clear();
    }

    fn check_destroyed(&self) -> Result<(), DeveloperError> {
        if self.destroyed {
            Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ))
        } else {
            Ok(())
        }
    }
}
