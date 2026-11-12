use cesium_math::DeveloperError;

use super::create_command::{create_command, Command};

/// Options for creating a `ProviderViewModel`.
#[derive(Clone, Debug)]
pub struct ProviderViewModelOptions<R: Clone + 'static = ()> {
    pub name: Option<String>,
    pub tooltip: Option<String>,
    pub icon_url: Option<String>,
    pub category: Option<String>,
    pub creation_command: Option<Command<(), R>>,
}

impl<R: Clone + 'static> Default for ProviderViewModelOptions<R> {
    fn default() -> Self {
        Self {
            name: None,
            tooltip: None,
            icon_url: None,
            category: None,
            creation_command: None,
        }
    }
}

/// A view model representing each item in the `BaseLayerPicker`.
#[derive(Clone, Debug)]
pub struct ProviderViewModel<R: Clone + 'static = ()> {
    name: String,
    tooltip: String,
    icon_url: String,
    category: String,
    creation_command: Command<(), R>,
}

impl<R: Clone + 'static> ProviderViewModel<R> {
    /// Creates a new `ProviderViewModel` with the specified options.
    /// Returns `Err(DeveloperError)` if any required field is missing.
    pub fn new(options: ProviderViewModelOptions<R>) -> Result<Self, DeveloperError> {
        let name = options
            .name
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("options.name is required."))?;
        let tooltip = options
            .tooltip
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("options.tooltip is required."))?;
        let icon_url = options
            .icon_url
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("options.iconUrl is required."))?;
        let creation_command = options
            .creation_command
            .ok_or_else(|| DeveloperError::new("options.creationFunction is required."))?;

        Ok(Self {
            name,
            tooltip,
            icon_url,
            category: options.category.unwrap_or_default(),
            creation_command,
        })
    }

    /// Convenience constructor taking name, tooltip, icon_url and a creation closure.
    pub fn from_fn<F>(
        name: impl Into<String>,
        tooltip: impl Into<String>,
        icon_url: impl Into<String>,
        creation_fn: F,
    ) -> Result<Self, DeveloperError>
    where
        F: Fn(()) -> R + 'static,
    {
        Self::new(ProviderViewModelOptions {
            name: Some(name.into()),
            tooltip: Some(tooltip.into()),
            icon_url: Some(icon_url.into()),
            category: None,
            creation_command: Some(create_command(creation_fn)),
        })
    }

    /// Gets the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the tooltip.
    pub fn tooltip(&self) -> &str {
        &self.tooltip
    }

    /// Gets the icon URL.
    pub fn icon_url(&self) -> &str {
        &self.icon_url
    }

    /// Gets the category.
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Gets the creation command.
    pub fn creation_command(&self) -> &Command<(), R> {
        &self.creation_command
    }
}
