use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

use cesium_math::{DeveloperError, Event};

use super::{
    create_command::{create_command, Command},
    provider_view_model::ProviderViewModel,
};

/// Represents a grouped category of provider view models.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderCategory<R: Clone + 'static = ()> {
    pub name: String,
    pub providers: Vec<ProviderViewModel<R>>,
}

/// A globe model interface used by BaseLayerPickerViewModel to update layers and terrain.
pub trait GlobeModel {
    fn set_depth_test_against_terrain(&mut self, enabled: bool);
    fn depth_test_against_terrain(&self) -> bool;
    fn terrain_provider_changed(&self) -> &Event<()>;
}

/// A default standalone/mock globe model for headless environments and tests.
#[derive(Clone, Debug, Default)]
pub struct StandaloneGlobe {
    pub depth_test_against_terrain: bool,
    pub terrain_provider_changed: Event<()>,
}

impl StandaloneGlobe {
    pub fn new() -> Self {
        Self {
            depth_test_against_terrain: false,
            terrain_provider_changed: Event::new(),
        }
    }
}

impl GlobeModel for StandaloneGlobe {
    fn set_depth_test_against_terrain(&mut self, enabled: bool) {
        self.depth_test_against_terrain = enabled;
    }

    fn depth_test_against_terrain(&self) -> bool {
        self.depth_test_against_terrain
    }

    fn terrain_provider_changed(&self) -> &Event<()> {
        &self.terrain_provider_changed
    }
}

/// Construction options for `BaseLayerPickerViewModel`.
#[derive(Clone)]
pub struct BaseLayerPickerViewModelOptions<G: GlobeModel + 'static, R: Clone + 'static = ()> {
    pub globe: Option<Rc<RefCell<G>>>,
    pub imagery_provider_view_models: Vec<ProviderViewModel<R>>,
    pub selected_imagery_provider_view_model: Option<ProviderViewModel<R>>,
    pub terrain_provider_view_models: Vec<ProviderViewModel<R>>,
    pub selected_terrain_provider_view_model: Option<ProviderViewModel<R>>,
}

impl<G: GlobeModel + 'static, R: Clone + 'static> Default for BaseLayerPickerViewModelOptions<G, R> {
    fn default() -> Self {
        Self {
            globe: None,
            imagery_provider_view_models: Vec::new(),
            selected_imagery_provider_view_model: None,
            terrain_provider_view_models: Vec::new(),
            selected_terrain_provider_view_model: None,
        }
    }
}

struct InnerBaseLayerState<G: GlobeModel + 'static, R: Clone + 'static> {
    globe: Rc<RefCell<G>>,
    imagery_providers: RefCell<Vec<ProviderViewModel<R>>>,
    terrain_providers: RefCell<Vec<ProviderViewModel<R>>>,
    selected_imagery: RefCell<Option<ProviderViewModel<R>>>,
    selected_terrain: RefCell<Option<ProviderViewModel<R>>>,
    drop_down_visible: Cell<bool>,
}

/// View model for the `BaseLayerPicker` widget.
#[derive(Clone)]
pub struct BaseLayerPickerViewModel<G: GlobeModel + 'static = StandaloneGlobe, R: Clone + 'static = ()> {
    inner: Rc<InnerBaseLayerState<G, R>>,
    toggle_drop_down_cmd: Command<(), ()>,
}

impl<G: GlobeModel + 'static, R: Clone + 'static> BaseLayerPickerViewModel<G, R> {
    /// Creates a new `BaseLayerPickerViewModel`.
    pub fn new(options: BaseLayerPickerViewModelOptions<G, R>) -> Result<Self, DeveloperError> {
        let globe = options
            .globe
            .ok_or_else(|| DeveloperError::new("globe is required"))?;

        let initial_imagery = options
            .selected_imagery_provider_view_model
            .or_else(|| options.imagery_provider_view_models.first().cloned());
        let initial_terrain = options.selected_terrain_provider_view_model;

        let inner = Rc::new(InnerBaseLayerState {
            globe,
            imagery_providers: RefCell::new(options.imagery_provider_view_models),
            terrain_providers: RefCell::new(options.terrain_provider_view_models),
            selected_imagery: RefCell::new(None),
            selected_terrain: RefCell::new(None),
            drop_down_visible: Cell::new(false),
        });

        let inner_toggle = Rc::clone(&inner);
        let toggle_drop_down_cmd = create_command(move |()| {
            inner_toggle
                .drop_down_visible
                .set(!inner_toggle.drop_down_visible.get());
        });

        let vm = Self {
            inner,
            toggle_drop_down_cmd,
        };

        if let Some(imagery) = initial_imagery {
            vm.set_selected_imagery(Some(imagery));
        }
        if let Some(terrain) = initial_terrain {
            vm.set_selected_terrain(Some(terrain));
        }

        Ok(vm)
    }

    /// Gets the globe reference.
    pub fn globe(&self) -> Rc<RefCell<G>> {
        Rc::clone(&self.inner.globe)
    }

    /// Gets available imagery provider view models.
    pub fn imagery_provider_view_models(&self) -> Vec<ProviderViewModel<R>> {
        self.inner.imagery_providers.borrow().clone()
    }

    /// Sets available imagery provider view models.
    pub fn set_imagery_provider_view_models(&self, providers: Vec<ProviderViewModel<R>>) {
        *self.inner.imagery_providers.borrow_mut() = providers;
    }

    /// Gets available terrain provider view models.
    pub fn terrain_provider_view_models(&self) -> Vec<ProviderViewModel<R>> {
        self.inner.terrain_providers.borrow().clone()
    }

    /// Sets available terrain provider view models.
    pub fn set_terrain_provider_view_models(&self, providers: Vec<ProviderViewModel<R>>) {
        *self.inner.terrain_providers.borrow_mut() = providers;
    }

    /// Groups imagery providers into categories preserving encounter order.
    pub fn categorized_imagery_providers(&self) -> Vec<ProviderCategory<R>> {
        Self::group_into_categories(&self.inner.imagery_providers.borrow())
    }

    /// Groups terrain providers into categories preserving encounter order.
    pub fn categorized_terrain_providers(&self) -> Vec<ProviderCategory<R>> {
        Self::group_into_categories(&self.inner.terrain_providers.borrow())
    }

    fn group_into_categories(providers: &[ProviderViewModel<R>]) -> Vec<ProviderCategory<R>> {
        let mut map: BTreeMap<String, Vec<ProviderViewModel<R>>> = BTreeMap::new();
        let mut order = Vec::new();

        for p in providers {
            let cat = p.category().to_string();
            if !map.contains_key(&cat) {
                order.push(cat.clone());
            }
            map.entry(cat).or_default().push(p.clone());
        }

        order
            .into_iter()
            .map(|name| {
                let list = map.remove(&name).unwrap_or_default();
                ProviderCategory {
                    name,
                    providers: list,
                }
            })
            .collect()
    }

    /// Gets whether the dropdown is currently visible.
    pub fn drop_down_visible(&self) -> bool {
        self.inner.drop_down_visible.get()
    }

    /// Sets whether the dropdown is visible.
    pub fn set_drop_down_visible(&self, visible: bool) {
        self.inner.drop_down_visible.set(visible);
    }

    /// Gets the toggle dropdown command.
    pub fn toggle_drop_down(&self) -> &Command<(), ()> {
        &self.toggle_drop_down_cmd
    }

    /// Gets the currently selected imagery provider.
    pub fn selected_imagery(&self) -> Option<ProviderViewModel<R>> {
        self.inner.selected_imagery.borrow().clone()
    }

    /// Sets currently selected imagery provider. Closes dropdown and executes layer creation.
    pub fn set_selected_imagery(&self, value: Option<ProviderViewModel<R>>) {
        if let Some(ref val) = value {
            let _ = val.creation_command().call();
        }
        *self.inner.selected_imagery.borrow_mut() = value;
        self.inner.drop_down_visible.set(false);
    }

    /// Gets the currently selected terrain provider.
    pub fn selected_terrain(&self) -> Option<ProviderViewModel<R>> {
        self.inner.selected_terrain.borrow().clone()
    }

    /// Sets currently selected terrain provider. Closes dropdown and executes terrain creation.
    pub fn set_selected_terrain(&self, value: Option<ProviderViewModel<R>>) {
        if let Some(ref val) = value {
            let _ = val.creation_command().call();
            // Ellipsoid vs regular terrain depthTestAgainstTerrain check
            let is_ellipsoid = val.name().to_lowercase().contains("ellipsoid");
            self.inner
                .globe
                .borrow_mut()
                .set_depth_test_against_terrain(!is_ellipsoid);
        }
        *self.inner.selected_terrain.borrow_mut() = value;
        self.inner.drop_down_visible.set(false);
    }

    /// Gets composite button tooltip string.
    pub fn button_tooltip(&self) -> Option<String> {
        let sel_img = self.inner.selected_imagery.borrow();
        let sel_trn = self.inner.selected_terrain.borrow();

        let img_tip = sel_img.as_ref().map(|p| p.name());
        let trn_tip = sel_trn.as_ref().map(|p| p.name());

        match (img_tip, trn_tip) {
            (Some(img), Some(trn)) => Some(format!("{img}\n{trn}")),
            (Some(img), None) => Some(img.to_string()),
            (None, Some(trn)) => Some(trn.to_string()),
            (None, None) => None,
        }
    }

    /// Gets button icon URL from current selected imagery provider.
    pub fn button_image_url(&self) -> Option<String> {
        self.inner
            .selected_imagery
            .borrow()
            .as_ref()
            .map(|p| p.icon_url().to_string())
    }
}
