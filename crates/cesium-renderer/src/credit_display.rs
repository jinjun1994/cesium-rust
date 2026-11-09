//! CreditDisplay matching CesiumJS `Scene/CreditDisplay.js` (Apache-2.0).

use cesium_math::{Credit, DeveloperError};

pub const DEFAULT_DELIMITER: &str = ", ";

/// A display manager that collects and formats attributions / credits for on-screen and lightbox presentation.
#[derive(Debug, Clone, PartialEq)]
pub struct CreditDisplay {
    /// Delimiter string used to separate displayed credits.
    pub delimiter: String,
    static_credits: Vec<Credit>,
    current_frame_credits: Vec<Credit>,
    rendered_credits: Vec<Credit>,
    is_destroyed: bool,
}

impl Default for CreditDisplay {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CreditDisplay {
    /// Creates a new `CreditDisplay`.
    pub fn new(delimiter: Option<String>) -> Self {
        Self {
            delimiter: delimiter.unwrap_or_else(|| DEFAULT_DELIMITER.to_string()),
            static_credits: Vec::new(),
            current_frame_credits: Vec::new(),
            rendered_credits: Vec::new(),
            is_destroyed: false,
        }
    }

    /// Adds a credit to be displayed in the next frame.
    pub fn add_credit_to_next_frame(&mut self, credit: Credit) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if !self.current_frame_credits.contains(&credit) {
            self.current_frame_credits.push(credit);
        }
        Ok(())
    }

    /// Adds a static credit that will always be displayed across frames.
    pub fn add_static_credit(&mut self, credit: Credit) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if !self.static_credits.contains(&credit) {
            self.static_credits.push(credit);
        }
        Ok(())
    }

    /// Removes a static credit.
    pub fn remove_static_credit(&mut self, credit: &Credit) -> bool {
        if self.is_destroyed {
            return false;
        }

        if let Some(pos) = self.static_credits.iter().position(|c| c == credit) {
            self.static_credits.remove(pos);
            true
        } else {
            false
        }
    }

    /// Resets dynamic frame credits at the start of a frame.
    pub fn begin_frame(&mut self) {
        self.current_frame_credits.clear();
    }

    /// Commits and aggregates credits at the end of a frame.
    pub fn end_frame(&mut self) {
        self.rendered_credits.clear();

        for credit in &self.static_credits {
            if !self.rendered_credits.contains(credit) {
                self.rendered_credits.push(credit.clone());
            }
        }

        for credit in &self.current_frame_credits {
            if !self.rendered_credits.contains(credit) {
                self.rendered_credits.push(credit.clone());
            }
        }
    }

    /// Gets all current credits intended for on-screen display.
    pub fn screen_credits(&self) -> Vec<&Credit> {
        self.rendered_credits
            .iter()
            .filter(|c| c.show_on_screen())
            .collect()
    }

    /// Gets all current credits intended for lightbox display.
    pub fn lightbox_credits(&self) -> Vec<&Credit> {
        self.rendered_credits
            .iter()
            .filter(|c| !c.show_on_screen())
            .collect()
    }

    /// Formats all on-screen credits joined by the configured delimiter.
    pub fn formatted_screen_credits(&self) -> String {
        let credits = self.screen_credits();
        credits
            .iter()
            .map(|c| c.html())
            .collect::<Vec<_>>()
            .join(&self.delimiter)
    }

    /// Formats all lightbox credits joined by the configured delimiter.
    pub fn formatted_lightbox_credits(&self) -> String {
        let credits = self.lightbox_credits();
        credits
            .iter()
            .map(|c| c.html())
            .collect::<Vec<_>>()
            .join(&self.delimiter)
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.static_credits.clear();
        self.current_frame_credits.clear();
        self.rendered_credits.clear();
        self.is_destroyed = true;
    }
}
