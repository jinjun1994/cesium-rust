//! I3SBuildingSceneLayerExplorerViewModel matching CesiumJS `Widgets/I3SBuildingSceneLayerExplorer/I3SBuildingSceneLayerExplorerViewModel.js` (Apache-2.0).

use cesium_math::DeveloperError;
use cesium_renderer::i3s_data_provider::{I3SAttributeFilter, I3SSublayer};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Represents a building level in the explorer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum I3SLevel {
    All,
    Level(i64),
    Named(String),
}

impl fmt::Display for I3SLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "All"),
            Self::Level(num) => write!(f, "{num}"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

/// A top-level layer option in the building explorer selector dropdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopLayerItem {
    pub name: String,
    pub model_name: Option<String>,
    pub disabled: bool,
    pub index: i32,
}

impl TopLayerItem {
    /// Creates a new `TopLayerItem`.
    pub fn new(
        name: impl Into<String>,
        model_name: Option<String>,
        disabled: bool,
        index: i32,
    ) -> Self {
        Self {
            name: name.into(),
            model_name,
            disabled,
            index,
        }
    }

    /// Returns true if this top layer represents the overview model.
    pub fn is_overview(&self) -> bool {
        self.model_name.as_deref() == Some("Overview")
    }

    /// Returns true if this top layer represents the full model.
    pub fn is_full_model(&self) -> bool {
        self.model_name.as_deref() == Some("FullModel")
    }

    /// Returns true if this top layer is an overview or full model.
    pub fn is_top_layer(&self) -> bool {
        self.is_overview() || self.is_full_model()
    }
}

/// Abstract data provider source interface for I3S building scene layers.
pub trait I3SDataProviderSource {
    fn sublayers(&self) -> &[I3SSublayer];
    fn sublayers_mut(&mut self) -> &mut [I3SSublayer];
    fn get_attribute_names(&self) -> Vec<String>;
    fn get_attribute_values(&self, name: &str) -> Vec<Value>;
    fn filter_by_attributes(&mut self, filters: Option<&[I3SAttributeFilter]>);
    fn show(&self) -> bool {
        true
    }
    fn set_show(&mut self, _show: bool) {}
}

impl I3SDataProviderSource for cesium_renderer::i3s_data_provider::I3SDataProvider {
    fn sublayers(&self) -> &[I3SSublayer] {
        self.sublayers()
    }

    fn sublayers_mut(&mut self) -> &mut [I3SSublayer] {
        self.sublayers_mut()
    }

    fn get_attribute_names(&self) -> Vec<String> {
        self.get_attribute_names()
    }

    fn get_attribute_values(&self, name: &str) -> Vec<Value> {
        self.get_attribute_values(name)
    }

    fn filter_by_attributes(&mut self, filters: Option<&[I3SAttributeFilter]>) {
        self.filter_by_attributes(filters);
    }

    fn show(&self) -> bool {
        self.show()
    }

    fn set_show(&mut self, show: bool) {
        self.set_show(show);
    }
}

/// In-memory mock/simple I3S data provider for testing and standalone scenarios.
#[derive(Clone, Debug, Default)]
pub struct SimpleI3SDataProvider {
    pub sublayers: Vec<I3SSublayer>,
    pub attribute_names: Vec<String>,
    pub attribute_values: HashMap<String, Vec<Value>>,
    pub show: bool,
    pub last_filters: Option<Vec<I3SAttributeFilter>>,
    pub filter_by_attributes_calls: usize,
}

impl SimpleI3SDataProvider {
    /// Creates a new `SimpleI3SDataProvider`.
    pub fn new(sublayers: Vec<I3SSublayer>) -> Self {
        Self {
            sublayers,
            attribute_names: Vec::new(),
            attribute_values: HashMap::new(),
            show: true,
            last_filters: None,
            filter_by_attributes_calls: 0,
        }
    }

    /// Sets attribute values for a given attribute name.
    pub fn set_attribute_values(&mut self, name: impl Into<String>, values: Vec<Value>) {
        let n = name.into();
        if !self.attribute_names.contains(&n) {
            self.attribute_names.push(n.clone());
        }
        self.attribute_values.insert(n, values);
    }
}

impl I3SDataProviderSource for SimpleI3SDataProvider {
    fn sublayers(&self) -> &[I3SSublayer] {
        &self.sublayers
    }

    fn sublayers_mut(&mut self) -> &mut [I3SSublayer] {
        &mut self.sublayers
    }

    fn get_attribute_names(&self) -> Vec<String> {
        self.attribute_names.clone()
    }

    fn get_attribute_values(&self, name: &str) -> Vec<Value> {
        self.attribute_values.get(name).cloned().unwrap_or_default()
    }

    fn filter_by_attributes(&mut self, filters: Option<&[I3SAttributeFilter]>) {
        self.filter_by_attributes_calls += 1;
        self.last_filters = filters.map(|f| f.to_vec());
    }

    fn show(&self) -> bool {
        self.show
    }

    fn set_show(&mut self, show: bool) {
        self.show = show;
    }
}

fn is_overview(layer: &I3SSublayer) -> bool {
    layer.model_name == "Overview"
}

fn is_full_model(layer: &I3SSublayer) -> bool {
    layer.model_name == "FullModel"
}

fn is_top_layer(layer: &I3SSublayer) -> bool {
    is_overview(layer) || is_full_model(layer)
}

fn add_top_layer(
    mut layer: I3SSublayer,
    top_layers: &mut Vec<TopLayerItem>,
    sublayers: &mut Vec<I3SSublayer>,
) -> Option<TopLayerItem> {
    if is_top_layer(&layer) {
        layer.visibility = false;
        for sub in &mut layer.sublayers {
            sub.visibility = true;
        }

        let item = TopLayerItem::new(
            layer.name.clone(),
            Some(layer.model_name.clone()),
            false,
            sublayers.len() as i32,
        );
        top_layers.push(item.clone());
        sublayers.push(layer);
        Some(item)
    } else {
        None
    }
}

/// The view model for `I3SBuildingSceneLayerExplorer`.
pub struct I3SBuildingSceneLayerExplorerViewModel {
    provider: Rc<RefCell<dyn I3SDataProviderSource>>,
    sublayers: Vec<I3SSublayer>,
    levels: Vec<I3SLevel>,
    current_level: Option<I3SLevel>,
    selected_level: I3SLevel,
    top_layers: Vec<TopLayerItem>,
    current_layer: Option<TopLayerItem>,
    default_layer: Option<TopLayerItem>,
    has_synthetic_full_model: bool,
    is_bsl_wrapper_visible: bool,
    is_destroyed: bool,
}

impl I3SBuildingSceneLayerExplorerViewModel {
    /// Creates a new `I3SBuildingSceneLayerExplorerViewModel` instance.
    pub fn new(
        provider: Rc<RefCell<dyn I3SDataProviderSource>>,
    ) -> Result<Self, DeveloperError> {
        let mut top_layers = vec![TopLayerItem::new(
            "Select a layer to explore...",
            None,
            true,
            -1,
        )];
        let mut sublayers = Vec::new();
        let mut default_layer: Option<TopLayerItem> = None;

        let prov_sublayers = provider.borrow().sublayers().to_vec();
        for sub in prov_sublayers {
            if let Some(top_item) = add_top_layer(sub, &mut top_layers, &mut sublayers) {
                if top_item.is_overview()
                    || (default_layer.is_none() && top_item.is_full_model())
                {
                    default_layer = Some(top_item);
                }
            }
        }

        let mut has_synthetic_full_model = false;
        if top_layers.len() == 1 && !provider.borrow().sublayers().is_empty() {
            provider.borrow_mut().set_show(false);
            let full_model = I3SSublayer {
                name: "Full Model".to_string(),
                model_name: "FullModel".to_string(),
                visibility: provider.borrow().show(),
                sublayers: provider.borrow().sublayers().to_vec(),
                ..Default::default()
            };
            default_layer = add_top_layer(full_model, &mut top_layers, &mut sublayers);
            has_synthetic_full_model = true;
        } else if top_layers.len() == 1 {
            top_layers[0].name = "Building layers not found".to_string();
        }

        // Set levels
        let mut raw_levels: Vec<i64> = Vec::new();
        {
            let prov = provider.borrow();
            let attr_names = prov.get_attribute_names();
            for attr in attr_names {
                if attr == "BldgLevel" {
                    let vals = prov.get_attribute_values(&attr);
                    for v in vals {
                        if let Some(n) = v.as_i64() {
                            raw_levels.push(n);
                        }
                    }
                }
            }
        }
        raw_levels.sort_unstable();
        let mut levels = vec![I3SLevel::All];
        for lvl in raw_levels {
            levels.push(I3SLevel::Level(lvl));
        }

        let is_bsl_wrapper_visible = default_layer
            .as_ref()
            .map(|l| l.is_full_model())
            .unwrap_or(false);

        Ok(Self {
            provider,
            sublayers,
            levels,
            current_level: None,
            selected_level: I3SLevel::All,
            top_layers,
            current_layer: None,
            default_layer,
            has_synthetic_full_model,
            is_bsl_wrapper_visible,
            is_destroyed: false,
        })
    }

    /// Gets the collection of top sublayers.
    pub fn sublayers(&self) -> &[I3SSublayer] {
        &self.sublayers
    }

    /// Gets mutable sublayers.
    pub fn sublayers_mut(&mut self) -> &mut [I3SSublayer] {
        &mut self.sublayers
    }

    /// Gets available building levels.
    pub fn levels(&self) -> &[I3SLevel] {
        &self.levels
    }

    /// Gets the current building level.
    pub fn current_level(&self) -> Option<&I3SLevel> {
        self.current_level.as_ref()
    }

    /// Sets the current building level and dispatches attribute filtering.
    pub fn set_current_level(&mut self, level: I3SLevel) {
        self.current_level = Some(level.clone());
        let mut prov = self.provider.borrow_mut();
        match &level {
            I3SLevel::All => {
                prov.filter_by_attributes(None);
            }
            I3SLevel::Level(num) => {
                let filter = I3SAttributeFilter::new(
                    "BldgLevel",
                    vec![Value::from(*num)],
                );
                prov.filter_by_attributes(Some(&[filter]));
            }
            I3SLevel::Named(name) => {
                let filter = I3SAttributeFilter::new(
                    "BldgLevel",
                    vec![Value::String(name.clone())],
                );
                prov.filter_by_attributes(Some(&[filter]));
            }
        }
    }

    /// Gets the selected building level.
    pub fn selected_level(&self) -> &I3SLevel {
        &self.selected_level
    }

    /// Sets the selected building level.
    pub fn set_selected_level(&mut self, level: I3SLevel) {
        self.selected_level = level;
    }

    /// Gets available top layers.
    pub fn top_layers(&self) -> &[TopLayerItem] {
        &self.top_layers
    }

    /// Gets the current top layer.
    pub fn current_layer(&self) -> Option<&TopLayerItem> {
        self.current_layer.as_ref()
    }

    /// Sets the current top layer and manages sublayer visibilities.
    pub fn set_current_layer(&mut self, layer: TopLayerItem) {
        if layer.is_top_layer() {
            for sub in &mut self.sublayers {
                sub.visibility = false;
            }
            if layer.index >= 0 && (layer.index as usize) < self.sublayers.len() {
                self.sublayers[layer.index as usize].visibility = true;
            }

            if layer.is_full_model() {
                let sel = self.selected_level.clone();
                self.set_current_level(sel);
                self.is_bsl_wrapper_visible = true;
                if self.has_synthetic_full_model {
                    self.provider.borrow_mut().set_show(true);
                }
            } else {
                if let Some(curr) = self.current_level.clone() {
                    self.selected_level = curr;
                }
                self.set_current_level(I3SLevel::All);
                self.is_bsl_wrapper_visible = false;
                if self.has_synthetic_full_model {
                    self.provider.borrow_mut().set_show(false);
                }
            }
        }
        self.current_layer = Some(layer);
    }

    /// Gets the default top layer.
    pub fn default_layer(&self) -> Option<&TopLayerItem> {
        self.default_layer.as_ref()
    }

    /// Returns whether the BSL wrapper is visible.
    pub fn is_bsl_wrapper_visible(&self) -> bool {
        self.is_bsl_wrapper_visible
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this object and releases resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.sublayers.clear();
        self.top_layers.clear();
        self.levels.clear();
    }
}
