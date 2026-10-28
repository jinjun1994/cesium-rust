//! Constant screen placement and distance properties shared by labels/billboards/points.
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HorizontalOrigin {
    Left,
    #[default]
    Center,
    Right,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerticalOrigin {
    Top,
    #[default]
    Center,
    Bottom,
    Baseline,
}
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ScreenOptions {
    pub distance_display_condition: Option<[f64; 2]>,
    pub scale_by_distance: Option<[f64; 4]>,
    pub translucency_by_distance: Option<[f64; 4]>,
    pub pixel_offset_scale_by_distance: Option<[f64; 4]>,
    pub disable_depth_test_distance: f64,
    pub rotation: f64,
    pub horizontal_origin: HorizontalOrigin,
    pub vertical_origin: VerticalOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "eyeOffset")]
    pub eye_offset: Option<[f64; 3]>,
}
impl ScreenOptions {
    pub fn distance(&self) -> cesium_renderer::ScreenDistance {
        cesium_renderer::ScreenDistance {
            display: self.distance_display_condition,
            scale: self.scale_by_distance,
            translucency: self.translucency_by_distance,
            offset: self.pixel_offset_scale_by_distance,
            disable_depth_test_distance: self.disable_depth_test_distance,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.rotation.is_finite() || self.rotation.abs() > 10000. {
            return Err("finite billboard rotation <=10000 radians required".into());
        }
        if let Some(eo) = self.eye_offset {
            if eo.iter().any(|v| !v.is_finite() || v.abs() > 1e7) {
                return Err("finite eyeOffset required".into());
            }
        }
        self.distance().validate().map_err(|e| e.to_string())
    }
}
pub(crate) fn array<const N: usize>(v: &Value, key: &str) -> Result<[f64; N], String> {
    let a = if v.is_object() {
        super::entities::known(v, &[key])?;
        v[key].as_array()
    } else {
        v.as_array()
    }
    .ok_or("constant distance array required")?;
    if a.len() != N {
        return Err("distance array length".into());
    }
    let mut out = [0.; N];
    for (i, x) in a.iter().enumerate() {
        out[i] = super::entities::number(x)?;
    }
    Ok(out)
}
pub fn czml(g: &Value) -> Result<ScreenOptions, String> {
    let enum_value = |v: &Value, key: &str| -> Result<Value, String> {
        if v.is_object() {
            super::entities::known(v, &[key])?;
            Ok(v[key].clone())
        } else {
            Ok(v.clone())
        }
    };
    let options = ScreenOptions {
        pixel_offset_scale_by_distance: g
            .get("pixelOffsetScaleByDistance")
            .map(|v| array(v, "nearFarScalar"))
            .transpose()?,
        disable_depth_test_distance: g
            .get("disableDepthTestDistance")
            .map(super::entities::number)
            .transpose()?
            .unwrap_or(0.),
        rotation: g
            .get("rotation")
            .map(super::entities::number)
            .transpose()?
            .unwrap_or(0.),
        distance_display_condition: g
            .get("distanceDisplayCondition")
            .map(|v| array(v, "distanceDisplayCondition"))
            .transpose()?,
        scale_by_distance: g
            .get("scaleByDistance")
            .map(|v| array(v, "nearFarScalar"))
            .transpose()?,
        translucency_by_distance: g
            .get("translucencyByDistance")
            .map(|v| array(v, "nearFarScalar"))
            .transpose()?,
        horizontal_origin: g
            .get("horizontalOrigin")
            .map(|v| {
                enum_value(v, "horizontalOrigin")
                    .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()))
            })
            .transpose()?
            .unwrap_or_default(),
        vertical_origin: g
            .get("verticalOrigin")
            .map(|v| {
                enum_value(v, "verticalOrigin")
                    .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()))
            })
            .transpose()?
            .unwrap_or_default(),
        eye_offset: if let Some(eo) = g.get("eyeOffset") {
            if let Some(c) = eo.get("cartesian") {
                let a = c.as_array().ok_or("cartesian eyeOffset array")?;
                if a.len() != 3 {
                    return Err("eyeOffset cartesian array length 3".into());
                }
                Some([
                    super::entities::number(&a[0])?,
                    super::entities::number(&a[1])?,
                    super::entities::number(&a[2])?,
                ])
            } else {
                None
            }
        } else {
            None
        },
    };
    options.validate()?;
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn czml_all_five_features_and_invalid_curves() {
        let value = serde_json::json!([{"id":"document","version":"1.0"},{"id":"label","position":{"cartographicDegrees":[110,25,1000]},"label":{"text":{"string":"地球 Globe"},"font":"24px sans-serif","fillColor":{"rgba":[255,255,255,255]},"horizontalOrigin":"LEFT","verticalOrigin":"TOP","distanceDisplayCondition":{"distanceDisplayCondition":[0,200000]},"scaleByDistance":{"nearFarScalar":[1000,2,100000,0.5]},"translucencyByDistance":{"nearFarScalar":[1000,1,100000,0]}}}]);
        let (entities, _) = crate::entities::czml(&serde_json::to_vec(&value).unwrap()).unwrap();
        let e = &entities[0];
        assert!(matches!(e.shape, crate::entities::Shape::Label { .. }));
        assert_eq!(e.screen.horizontal_origin, HorizontalOrigin::Left);
        assert_eq!(e.screen.vertical_origin, VerticalOrigin::Top);
        assert_eq!(e.screen.distance().evaluate(100000.), [0.5, 0., 1.]);
        let mut bad = value.clone();
        bad[1]["label"]["scaleByDistance"] = serde_json::json!({"nearFarScalar":[1000,1,1000,1]});
        assert!(crate::entities::czml(&serde_json::to_vec(&bad).unwrap()).is_err());
        bad = value.clone();
        bad[1]["label"]["scaleByDistance"]["interval"] = serde_json::json!("dynamic");
        assert!(crate::entities::czml(&serde_json::to_vec(&bad).unwrap()).is_err());
        bad = value;
        bad[1]["label"]["heightReference"] = serde_json::json!(2);
        assert!(crate::entities::czml(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    #[test]
    fn origins_and_screen_properties_reject_wrong_graphics() {
        let mut v = serde_json::json!([{"id":"box","positionDegrees":[110,25,1000],"shape":{"type":"box","dimensions":[1,1,1]},"screen":{"scaleByDistance":[0,1,100,0]}}]);
        assert!(crate::entities::decode(&serde_json::to_vec(&v).unwrap()).is_err());
        v[0]["shape"] = serde_json::json!({"type":"point","pixelSize":20});
        v[0]["screen"] = serde_json::json!({"horizontalOrigin":"LEFT"});
        assert!(crate::entities::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;
    #[test]
    fn czml_effects_and_wrong_graphics_rejection() {
        let bytes = include_bytes!("../../../tests/entities/screen-effects.czml");
        let url = url::Url::parse("file:///tests/screen-effects.czml").unwrap();
        let hydrated = crate::entity_resources::hydrate(bytes, &url, |_, _| {
            Ok(include_bytes!("../../../tests/entities/image-test.png").to_vec())
        })
        .unwrap();
        let (e, _) = crate::entities::czml(&hydrated).unwrap();
        assert_eq!(e.len(), 6);
        assert!(e[0].screen.rotation == 0.);
        assert!(e[2].screen.rotation > 1.5);
        assert_eq!(e[3].screen.pixel_offset_scale_by_distance.unwrap()[3], 3.);
        assert_eq!(e[4].screen.disable_depth_test_distance, 80000.);
        let mut v: Value =
            serde_json::from_slice(include_bytes!("../../../tests/entities/labels.json")).unwrap();
        v[0]["screen"]["rotation"] = serde_json::json!(1.);
        assert!(crate::entities::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}
