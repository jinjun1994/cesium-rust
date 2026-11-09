//! SharedContext and SceneContext matching CesiumJS `Renderer/SharedContext.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::context::{Context, ContextOptions};
use cesium_math::DeveloperError;

/// Options for creating a SharedContext.
#[derive(Clone, Debug, Default)]
pub struct SharedContextOptions {
    pub context_options: Option<ContextOptions>,
    pub auto_destroy: Option<bool>,
}

pub struct SharedContextInner {
    pub context: Context,
    pub canvases: Vec<String>,
    pub auto_destroy: bool,
    pub is_destroyed: bool,
}

/// Enables a single WebGL context to be shared by multiple scenes / canvases.
#[derive(Clone)]
pub struct SharedContext {
    pub inner: Rc<RefCell<SharedContextInner>>,
}

/// A scene-specific context proxy created from a SharedContext.
pub struct SceneContext {
    pub canvas: String,
    pub width: u32,
    pub height: u32,
    inner: Weak<RefCell<SharedContextInner>>,
    is_destroyed: bool,
}

impl SharedContext {
    /// Creates a new SharedContext instance.
    pub fn new(options: Option<SharedContextOptions>) -> Result<Self, DeveloperError> {
        let opts = options.unwrap_or_default();
        let auto_destroy = opts.auto_destroy.unwrap_or(true);
        let ctx = Context::new(opts.context_options.unwrap_or_default())?;

        let inner = SharedContextInner {
            context: ctx,
            canvases: Vec::new(),
            auto_destroy,
            is_destroyed: false,
        };

        Ok(Self {
            inner: Rc::new(RefCell::new(inner)),
        })
    }

    /// Creates a scene context for a given canvas ID and dimensions.
    pub fn create_scene_context(
        &self,
        canvas: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Result<SceneContext, DeveloperError> {
        let mut inner = self.inner.borrow_mut();
        if inner.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let canvas_name = canvas.into();
        if canvas_name.is_empty() {
            return Err(DeveloperError::new(
                "canvas used with SharedContext must provide a 2d context",
            ));
        }

        if inner.canvases.contains(&canvas_name) {
            return Err(DeveloperError::new(
                "canvas is already associated with a scene",
            ));
        }

        inner.canvases.push(canvas_name.clone());

        Ok(SceneContext {
            canvas: canvas_name,
            width,
            height,
            inner: Rc::downgrade(&self.inner),
            is_destroyed: false,
        })
    }

    /// Retrieves list of canvases currently registered.
    pub fn canvases(&self) -> Vec<String> {
        self.inner.borrow().canvases.clone()
    }

    /// Returns the drawing buffer dimensions of the underlying shared context.
    pub fn drawing_buffer_dimensions(&self) -> (u32, u32) {
        let inner = self.inner.borrow();
        (inner.context.drawing_buffer_width, inner.context.drawing_buffer_height)
    }

    /// Whether this shared context has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.borrow().is_destroyed
    }

    /// Destroys the shared context and releases all underlying resources.
    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        let mut inner = self.inner.borrow_mut();
        if inner.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        inner.is_destroyed = true;
        inner.canvases.clear();
        inner.context.destroy()?;
        Ok(())
    }
}

impl SceneContext {
    pub fn canvas(&self) -> Result<&str, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        Ok(&self.canvas)
    }

    pub fn drawing_buffer_width(&self) -> Result<u32, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        Ok(self.width)
    }

    pub fn drawing_buffer_height(&self) -> Result<u32, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        Ok(self.height)
    }

    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Begins a frame by ensuring the off-screen canvas is at least as large as the on-screen canvas.
    pub fn begin_frame(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if let Some(shared_rc) = self.inner.upgrade() {
            let mut shared = shared_rc.borrow_mut();
            if shared.context.drawing_buffer_width < self.width {
                shared.context.drawing_buffer_width = self.width;
            }
            if shared.context.drawing_buffer_height < self.height {
                shared.context.drawing_buffer_height = self.height;
            }
        }
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        Ok(())
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this scene context and removes its canvas association.
    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        self.is_destroyed = true;

        if let Some(shared_rc) = self.inner.upgrade() {
            let mut shared = shared_rc.borrow_mut();
            if let Some(pos) = shared.canvases.iter().position(|c| c == &self.canvas) {
                shared.canvases.remove(pos);
            }
            if shared.auto_destroy && shared.canvases.is_empty() {
                shared.is_destroyed = true;
                let _ = shared.context.destroy();
            }
        }
        Ok(())
    }
}
