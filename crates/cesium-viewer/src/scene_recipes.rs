//! Static web-scene recipe inputs. Native smoke exercises their shared globe/projection kernel.
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Overlay {
    pub position_degrees: [f64; 3],
    pub position_cartesian: [f64; 3],
    pub image: String,
}
#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum SceneRecipe {
    #[serde(rename = "offline")]
    Offline {
        template: String,
        #[serde(rename = "baseLayerPicker")]
        base_layer_picker: bool,
        geocoder: bool,
    },
    #[serde(rename = "hello-world")]
    HelloWorld {},
    #[serde(rename = "html-overlays")]
    HtmlOverlays { overlay: Overlay },
    #[serde(rename = "resolution-scaling")]
    ResolutionScaling {
        #[serde(rename = "resolutionScale")]
        resolution_scale: f64,
        #[serde(rename = "useBrowserRecommendedResolution")]
        use_browser_recommended_resolution: bool,
    },
}
pub fn decode(bytes: &[u8]) -> Result<SceneRecipe, String> {
    if bytes.len() > 16384 {
        return Err("scene input limit".into());
    }
    let v: SceneRecipe = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    match &v {
        SceneRecipe::Offline {
            template,
            base_layer_picker,
            geocoder,
        } if template != "../../assets/NaturalEarthII/{z}/{x}/{reverseY}.jpg"
            || *base_layer_picker
            || *geocoder =>
        {
            return Err("offline source/options mismatch".into())
        }
        SceneRecipe::HtmlOverlays { overlay } => {
            let c = cesium_math::Cartographic::degrees(
                overlay.position_degrees[0],
                overlay.position_degrees[1],
                overlay.position_degrees[2],
            )
            .map_err(|e| e.to_string())?;
            if c.to_ecef()
                .distance(glam::DVec3::from_array(overlay.position_cartesian))
                > 0.001
                || overlay.image != "Cesium_Logo_overlay.png"
            {
                return Err("overlay source position/image mismatch".into());
            }
        }
        SceneRecipe::ResolutionScaling {
            resolution_scale, ..
        } if !resolution_scale.is_finite() || !(0.1..=2.).contains(resolution_scale) => {
            return Err("resolution scale bound".into());
        }
        _ => {}
    }
    Ok(v)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn offline_provider() -> Result<crate::imagery::Provider, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/NaturalEarthII");
    let base = url::Url::from_directory_path(root.canonicalize().map_err(|e| e.to_string())?)
        .map_err(|_| "offline asset directory")?;
    crate::imagery::Provider::template(
        &format!("{base}{{z}}/{{x}}/{{reverseY}}.jpg"),
        crate::imagery::Projection::Geographic,
        2,
        "Natural Earth II · local offline imagery",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_scene_contracts() {
        for id in ["hello-world", "html-overlays", "resolution-scaling"] {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.scene.json"));
            assert!(decode(&std::fs::read(p).unwrap()).is_ok());
        }
        for input in [
            r#"{"type":"resolution-scaling","resolutionScale":0,"useBrowserRecommendedResolution":false}"#,
            r#"{"type":"hello-world","unexpected":true}"#,
            r#"{"type":"html-overlays","overlay":{"positionDegrees":[0,0,0],"positionCartesian":[0,0,0],"image":"Cesium_Logo_overlay.png"}}"#,
        ] {
            assert!(decode(input.as_bytes()).is_err());
        }
    }
    #[test]
    fn original_czml_positions_and_corridors() {
        for id in ["czml-position-definitions", "czml-corridor"] {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/sandcastle/full");
            let (actual, _) =
                crate::entities::czml(&std::fs::read(root.join(format!("{id}.czml"))).unwrap())
                    .unwrap();
            let expected: Vec<crate::entities::Entity> =
                serde_json::from_slice(&std::fs::read(root.join(format!("{id}.json"))).unwrap())
                    .unwrap();
            assert_eq!(actual.len(), expected.len());
            for (a, e) in actual.iter().zip(&expected) {
                assert_eq!(a.id, e.id);
                assert!(a.position(0.).unwrap().distance(e.position(0.).unwrap()) < 0.001);
                assert_eq!(
                    serde_json::to_value(&a.shape).unwrap(),
                    serde_json::to_value(&e.shape).unwrap()
                );
            }
        }
        assert!(crate::entities::czml(br#"[{"id":"document","version":"1.0"},{"id":"a","position":{"cartographicRadians":[0,0,0],"cartographicDegrees":[0,0,0]},"point":{}}]"#).is_err());
    }
}
