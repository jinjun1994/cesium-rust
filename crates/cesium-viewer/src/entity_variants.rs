//! Validated static menu alternatives. Each selection replaces, rather than accumulates, graphics.
use crate::{entities, RenderError, Scene};
use cesium_renderer::Renderer;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub label: String,
    pub entities: Vec<entities::Entity>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityVariants {
    pub variants: Vec<Variant>,
    pub initial: usize,
}
pub fn decode(bytes: &[u8]) -> Result<EntityVariants, String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("variant input limit".into());
    }
    let v: EntityVariants = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if v.variants.is_empty() || v.variants.len() > 32 || v.initial >= v.variants.len() {
        return Err("variant count/index".into());
    }
    let mut labels = std::collections::HashSet::new();
    for item in &v.variants {
        if item.label.is_empty() || item.label.len() > 256 || !labels.insert(&item.label) {
            return Err("unique bounded variant labels".into());
        }
        entities::decode(&serde_json::to_vec(&item.entities).map_err(|e| e.to_string())?)?;
    }
    Ok(v)
}
impl Scene {
    pub fn set_entity_variant(
        &mut self,
        r: &mut Renderer,
        bytes: &[u8],
        index: usize,
    ) -> Result<(), RenderError> {
        let v = decode(bytes).map_err(RenderError)?;
        let e = v
            .variants
            .get(index)
            .ok_or_else(|| RenderError("variant index".into()))?;
        self.set_entities(r, e.entities.clone())
    }
}
