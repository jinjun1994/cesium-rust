//! DataSourceDisplay matching CesiumJS `DataSources/DataSourceDisplay.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::{
    BillboardVisualizer, BoundingSphere, BoundingSphereState,
    Cesium3DTilesetVisualizer, CustomDataSource, DataSourceCollection, DeveloperError,
    Entity, EntityCluster, GeometryVisualizer, JulianDate, LabelVisualizer,
    ListenerId, ModelVisualizer, PathVisualizer, PointVisualizer, PolylineVisualizer,
    Visualizer,
};

/// Primitive collection used by DataSourceDisplay.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayPrimitiveCollection {
    pub id: String,
    items: Vec<String>,
}

impl DisplayPrimitiveCollection {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|item| item == id)
    }

    pub fn add(&mut self, id: impl Into<String>) {
        self.items.push(id.into());
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|item| item == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|s| s.as_str())
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// Simulated scene for DataSourceDisplay.
#[derive(Clone, Debug, Default)]
pub struct DataSourceDisplayScene {
    pub primitives: Rc<RefCell<DisplayPrimitiveCollection>>,
    pub ground_primitives: Rc<RefCell<DisplayPrimitiveCollection>>,
    pub request_render_mode: bool,
    pub render_requested: Rc<RefCell<bool>>,
}

impl DataSourceDisplayScene {
    pub fn new() -> Self {
        Self {
            primitives: Rc::new(RefCell::new(DisplayPrimitiveCollection::new("scene-primitives"))),
            ground_primitives: Rc::new(RefCell::new(DisplayPrimitiveCollection::new(
                "scene-ground-primitives",
            ))),
            request_render_mode: false,
            render_requested: Rc::new(RefCell::new(false)),
        }
    }

    pub fn request_render(&self) {
        *self.render_requested.borrow_mut() = true;
    }

    pub fn render_for_specs(&self) {
        *self.render_requested.borrow_mut() = false;
    }
}

pub type VisualizersCallback = Arc<
    dyn Fn(
            &DataSourceDisplayScene,
            Option<&EntityCluster>,
            &CustomDataSource,
        ) -> Vec<Box<dyn Visualizer>>
        + Send
        + Sync,
>;

pub type CustomVisualizerFactory =
    Arc<dyn Fn(&CustomDataSource) -> Box<dyn Visualizer> + Send + Sync>;

thread_local! {
    static REGISTERED_EXTRA_VISUALIZERS: RefCell<Vec<(&'static str, CustomVisualizerFactory)>> =
        const { RefCell::new(Vec::new()) };
}

/// Options for constructing DataSourceDisplay.
pub struct DataSourceDisplayOptions {
    pub scene: Option<DataSourceDisplayScene>,
    pub data_source_collection: Option<Rc<RefCell<DataSourceCollection>>>,
    pub visualizers_callback: Option<VisualizersCallback>,
}

/// Visualizes a collection of DataSource instances.
pub struct DataSourceDisplay {
    scene: DataSourceDisplayScene,
    data_source_collection: Rc<RefCell<DataSourceCollection>>,
    default_data_source: CustomDataSource,
    visualizers_callback: VisualizersCallback,
    visualizers_by_source: HashMap<String, Vec<Box<dyn Visualizer>>>,
    source_names: HashMap<String, String>,
    pub primitives: Rc<RefCell<DisplayPrimitiveCollection>>,
    pub ground_primitives: Rc<RefCell<DisplayPrimitiveCollection>>,
    primitives_added_to_scene: bool,
    ready: bool,
    is_destroyed: bool,
    source_added_sub: Option<ListenerId>,
    source_removed_sub: Option<ListenerId>,
    source_moved_sub: Option<ListenerId>,
}

impl DataSourceDisplay {
    /// Creates default visualizers for a data source.
    pub fn default_visualizers_callback(
        scene: &DataSourceDisplayScene,
        cluster: Option<&EntityCluster>,
        data_source: &CustomDataSource,
    ) -> Vec<Box<dyn Visualizer>> {
        let mut visualizers: Vec<Box<dyn Visualizer>> = Vec::with_capacity(8);

        let cluster_rc = cluster
            .map(|c| Rc::new(RefCell::new(c.clone())))
            .unwrap_or_else(|| Rc::new(RefCell::new(EntityCluster::default())));
        let entities_rc = Rc::new(RefCell::new(data_source.entities().clone()));

        // 1. BillboardVisualizer
        if let Ok(vis) = BillboardVisualizer::new(
            Some(Rc::clone(&cluster_rc)),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // 2. GeometryVisualizer
        if let Ok(vis) = GeometryVisualizer::new(
            Some(crate::GeometryVisualizerScene::new()),
            Some(Rc::clone(&entities_rc)),
            None,
            None,
        ) {
            visualizers.push(Box::new(vis));
        }

        // 3. LabelVisualizer
        if let Ok(vis) = LabelVisualizer::new(
            Some(Rc::clone(&cluster_rc)),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // 4. ModelVisualizer
        if let Ok(vis) = ModelVisualizer::new(
            Some(Rc::new(RefCell::new(crate::ModelVisualizerScene::new()))),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // 5. Cesium3DTilesetVisualizer
        if let Ok(vis) = Cesium3DTilesetVisualizer::new(
            Some(Rc::new(RefCell::new(crate::TilesetVisualizerScene::new()))),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // 6. PointVisualizer
        if let Ok(vis) = PointVisualizer::new(
            Some(Rc::clone(&cluster_rc)),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // 7. PathVisualizer
        if let Ok(vis) = PathVisualizer::new(
            Some(Rc::new(RefCell::new(crate::PathVisualizerScene::new()))),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // 8. PolylineVisualizer
        if let Ok(vis) = PolylineVisualizer::new(
            Some(Rc::new(RefCell::new(crate::PolylineVisualizerScene::new()))),
            Some(Rc::clone(&entities_rc)),
        ) {
            visualizers.push(Box::new(vis));
        }

        // Registered extra visualizers
        REGISTERED_EXTRA_VISUALIZERS.with(|reg| {
            for (_, factory) in reg.borrow().iter() {
                visualizers.push(factory(data_source));
            }
        });

        let _ = scene;
        visualizers
    }

    /// Registers a custom visualizer factory.
    pub fn register_visualizer(name: &'static str, factory: CustomVisualizerFactory) {
        REGISTERED_EXTRA_VISUALIZERS.with(|reg| {
            let mut reg = reg.borrow_mut();
            if !reg.iter().any(|(n, _)| *n == name) {
                reg.push((name, factory));
            }
        });
    }

    /// Unregisters a custom visualizer factory.
    pub fn unregister_visualizer(name: &'static str) {
        REGISTERED_EXTRA_VISUALIZERS.with(|reg| {
            reg.borrow_mut().retain(|(n, _)| *n != name);
        });
    }

    /// Creates a new DataSourceDisplay.
    pub fn new(options: Option<DataSourceDisplayOptions>) -> Result<Self, DeveloperError> {
        let options = options.ok_or_else(|| DeveloperError::new("options is required."))?;
        let scene = options
            .scene
            .ok_or_else(|| DeveloperError::new("scene is required."))?;
        let data_source_collection = options
            .data_source_collection
            .ok_or_else(|| DeveloperError::new("dataSourceCollection is required."))?;

        let callback = options.visualizers_callback.unwrap_or_else(|| {
            Arc::new(|scene, cluster, ds| {
                DataSourceDisplay::default_visualizers_callback(scene, cluster, ds)
            })
        });

        let default_ds = CustomDataSource::new("default");

        let primitives = Rc::new(RefCell::new(DisplayPrimitiveCollection::new(
            "display-primitives",
        )));
        let ground_primitives = Rc::new(RefCell::new(DisplayPrimitiveCollection::new(
            "display-ground-primitives",
        )));

        let mut display = Self {
            scene,
            data_source_collection,
            default_data_source: default_ds.clone(),
            visualizers_callback: callback,
            visualizers_by_source: HashMap::new(),
            source_names: HashMap::new(),
            primitives,
            ground_primitives,
            primitives_added_to_scene: false,
            ready: false,
            is_destroyed: false,
            source_added_sub: None,
            source_removed_sub: None,
            source_moved_sub: None,
        };

        // Initialize default data source visualizers and primitive
        display.on_data_source_added(&default_ds);

        // Check if data source collection already has sources
        let sources_count = display.data_source_collection.borrow().len();
        if sources_count > 0 {
            display.add_primitives_to_scene();
            let sources: Vec<CustomDataSource> = (0..sources_count)
                .filter_map(|i| display.data_source_collection.borrow().get(i).cloned())
                .collect();
            for ds in sources {
                display.on_data_source_added(&ds);
            }
        }

        // Listen for added / removed / moved events
        let added_sub = display
            .data_source_collection
            .borrow_mut()
            .data_source_added
            .add_listener(|_| {});
        let removed_sub = display
            .data_source_collection
            .borrow_mut()
            .data_source_removed
            .add_listener(|_| {});
        let moved_sub = display
            .data_source_collection
            .borrow_mut()
            .data_source_moved
            .add_listener(|_| {});

        display.source_added_sub = Some(added_sub);
        display.source_removed_sub = Some(removed_sub);
        display.source_moved_sub = Some(moved_sub);

        Ok(display)
    }

    pub fn scene(&self) -> &DataSourceDisplayScene {
        &self.scene
    }

    pub fn data_sources(&self) -> Rc<RefCell<DataSourceCollection>> {
        Rc::clone(&self.data_source_collection)
    }

    pub fn default_data_source(&self) -> &CustomDataSource {
        &self.default_data_source
    }

    pub fn default_data_source_mut(&mut self) -> &mut CustomDataSource {
        &mut self.default_data_source
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn add_primitives_to_scene(&mut self) {
        if !self.primitives_added_to_scene {
            self.scene
                .primitives
                .borrow_mut()
                .add(self.primitives.borrow().id.clone());
            self.scene
                .ground_primitives
                .borrow_mut()
                .add(self.ground_primitives.borrow().id.clone());
            self.primitives_added_to_scene = true;
        }
    }

    fn on_data_source_added(&mut self, ds: &CustomDataSource) {
        let visualizers = (self.visualizers_callback)(&self.scene, None, ds);
        self.visualizers_by_source
            .insert(ds.id().to_string(), visualizers);

        let name = if ds.name().is_empty() {
            ds.id().to_string()
        } else {
            ds.name().to_string()
        };
        self.source_names.insert(ds.id().to_string(), name.clone());

        self.primitives.borrow_mut().add(format!("{}-primitives", name));
        self.ground_primitives
            .borrow_mut()
            .add(format!("{}-ground-primitives", name));
    }

    fn on_data_source_removed(&mut self, ds_id: &str) {
        if let Some(mut vis_list) = self.visualizers_by_source.remove(ds_id) {
            for v in vis_list.iter_mut() {
                v.destroy();
            }
        }
        let name = self
            .source_names
            .remove(ds_id)
            .unwrap_or_else(|| ds_id.to_string());
        self.primitives
            .borrow_mut()
            .remove(&format!("{}-primitives", name));
        self.ground_primitives
            .borrow_mut()
            .remove(&format!("{}-ground-primitives", name));
    }

    /// Syncs data sources from data_source_collection.
    pub fn sync_data_sources(&mut self) {
        let current_ids: Vec<String> = {
            let col = self.data_source_collection.borrow();
            (0..col.len())
                .filter_map(|i| col.get(i).map(|ds| ds.id().to_string()))
                .collect()
        };

        // Add newly added data sources
        let new_sources: Vec<CustomDataSource> = {
            let col = self.data_source_collection.borrow();
            current_ids
                .iter()
                .filter(|id| !self.visualizers_by_source.contains_key(*id))
                .filter_map(|id| col.get_by_id(id).cloned())
                .collect()
        };
        for ds in new_sources {
            self.add_primitives_to_scene();
            self.on_data_source_added(&ds);
        }

        // Remove deleted data sources (excluding default)
        let existing_ids: Vec<String> = self.visualizers_by_source.keys().cloned().collect();
        for id in existing_ids {
            if id != self.default_data_source.id() && !current_ids.contains(&id) {
                self.on_data_source_removed(&id);
            }
        }

        // Reorder primitives to match collection order
        {
            let mut prims = self.primitives.borrow_mut();
            prims.clear();
            let def_name = if self.default_data_source.name().is_empty() {
                self.default_data_source.id().to_string()
            } else {
                self.default_data_source.name().to_string()
            };
            prims.add(format!("{}-primitives", def_name));
            let col = self.data_source_collection.borrow();
            for i in 0..col.len() {
                if let Some(ds) = col.get(i) {
                    let name = if ds.name().is_empty() {
                        ds.id().to_string()
                    } else {
                        ds.name().to_string()
                    };
                    prims.add(format!("{}-primitives", name));
                }
            }
        }

        // Check if entities were added to defaultDataSource
        if !self.default_data_source.entities().is_empty() {
            self.add_primitives_to_scene();
        }
    }

    /// Updates all visualizers for the given time.
    pub fn update(&mut self, time: Option<JulianDate>) -> Result<bool, DeveloperError> {
        let time = time.ok_or_else(|| DeveloperError::new("time is required."))?;

        self.sync_data_sources();

        let mut all_ready = true;
        for vis_list in self.visualizers_by_source.values_mut() {
            for vis in vis_list.iter_mut() {
                let r = vis.update(&time);
                if !r {
                    all_ready = false;
                }
            }
        }

        if all_ready && !self.ready {
            self.ready = true;
            if self.scene.request_render_mode {
                self.scene.request_render();
            }
        }

        Ok(self.ready)
    }

    /// Computes bounding sphere for entity.
    pub fn get_bounding_sphere(
        &self,
        entity: Option<&Entity>,
        allow_partial: Option<bool>,
        result: Option<&mut BoundingSphere>,
    ) -> Result<BoundingSphereState, DeveloperError> {
        let entity = entity.ok_or_else(|| DeveloperError::new("entity is required."))?;
        let allow_partial =
            allow_partial.ok_or_else(|| DeveloperError::new("allowPartial is required."))?;
        let result = result.ok_or_else(|| DeveloperError::new("result is required."))?;

        let mut computed_sphere: Option<BoundingSphere> = None;
        let mut has_done = false;
        let mut has_pending = false;

        for vis_list in self.visualizers_by_source.values() {
            for vis in vis_list {
                let mut tmp = BoundingSphere::default();
                let state = vis.get_bounding_sphere(entity, &mut tmp);
                match state {
                    BoundingSphereState::Done => {
                        has_done = true;
                        computed_sphere = match computed_sphere {
                            Some(cur) => Some(cur.union(tmp)),
                            None => Some(tmp),
                        };
                    }
                    BoundingSphereState::Pending => {
                        has_pending = true;
                    }
                    BoundingSphereState::Failed => {}
                }
            }
        }

        if has_done {
            if has_pending && !allow_partial {
                return Ok(BoundingSphereState::Pending);
            }
            if let Some(sphere) = computed_sphere {
                *result = sphere;
                return Ok(BoundingSphereState::Done);
            }
        }

        if has_pending {
            return Ok(BoundingSphereState::Pending);
        }

        Ok(BoundingSphereState::Failed)
    }

    /// Destroys the display.
    pub fn destroy(&mut self) {
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

        if let Some(sub) = self.source_added_sub.take() {
            self.data_source_collection
                .borrow_mut()
                .data_source_added
                .remove_listener(sub);
        }
        if let Some(sub) = self.source_removed_sub.take() {
            self.data_source_collection
                .borrow_mut()
                .data_source_removed
                .remove_listener(sub);
        }
        if let Some(sub) = self.source_moved_sub.take() {
            self.data_source_collection
                .borrow_mut()
                .data_source_moved
                .remove_listener(sub);
        }

        for mut vis_list in self.visualizers_by_source.drain().map(|(_, v)| v) {
            for v in vis_list.iter_mut() {
                v.destroy();
            }
        }

        self.primitives.borrow_mut().clear();
        self.ground_primitives.borrow_mut().clear();
    }
}

impl Drop for DataSourceDisplay {
    fn drop(&mut self) {
        self.destroy();
    }
}
