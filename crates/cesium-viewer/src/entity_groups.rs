//! Flat, geometry-free entity folders. Parent visibility never overwrites a child's own show flag.
use crate::{entities, RenderError, Scene};
use cesium_renderer::Renderer;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityGroup {
    pub id: String,
    pub name: String,
    pub show: bool,
    pub entity_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupedEntities {
    pub entities: Vec<entities::Entity>,
    pub groups: Vec<EntityGroup>,
}
impl Scene {
    pub fn set_grouped_entities(
        &mut self,
        r: &mut Renderer,
        bytes: &[u8],
    ) -> Result<(), RenderError> {
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(RenderError("grouped input limit".into()));
        }
        let input: GroupedEntities =
            serde_json::from_slice(bytes).map_err(|e| RenderError(e.to_string()))?;
        let mut group_ids = std::collections::HashSet::new();
        let mut members = std::collections::HashSet::new();
        let entity_ids: std::collections::HashSet<_> =
            input.entities.iter().map(|e| e.id.as_str()).collect();
        if input.groups.len() > 256 {
            return Err(RenderError("group count limit".into()));
        }
        for g in &input.groups {
            if g.id.is_empty()
                || g.id.len() > 256
                || g.name.len() > 1024
                || !group_ids.insert(g.id.as_str())
                || entity_ids.contains(g.id.as_str())
            {
                return Err(RenderError("unique bounded group IDs required".into()));
            }
            for id in &g.entity_ids {
                if !entity_ids.contains(id.as_str()) || !members.insert(id.as_str()) {
                    return Err(RenderError(
                        "group members must exist and have one parent".into(),
                    ));
                }
            }
        }
        self.set_entities(r, input.entities)?;
        self.entity_groups = input.groups;
        self.tick(0.);
        Ok(())
    }
    pub fn entity_groups(&self) -> &[EntityGroup] {
        &self.entity_groups
    }
    pub(crate) fn entity_group_visible(&self, id: &str) -> bool {
        self.entity_groups
            .iter()
            .all(|g| g.show || !g.entity_ids.iter().any(|member| member == id))
    }
    pub fn show_entity_group(&mut self, id: &str, show: bool) -> bool {
        let Some(g) = self.entity_groups.iter_mut().find(|g| g.id == id) else {
            return false;
        };
        g.show = show;
        let stop = !show
            && self
                .camera_control
                .tracked
                .as_ref()
                .is_some_and(|(id, _)| g.entity_ids.contains(id));
        if stop {
            self.stop_tracking();
        }
        self.tick(0.);
        true
    }
}
