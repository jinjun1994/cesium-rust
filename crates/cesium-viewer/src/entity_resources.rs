//! Bounded native resource hydration, before atomic Entity/CZML parsing and upload.
use base64::Engine;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;
pub fn hydrate<F>(bytes: &[u8], base: &Url, mut read: F) -> Result<Vec<u8>, String>
where
    F: FnMut(&Url, usize) -> Result<Vec<u8>, String>,
{
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("input byte limit".into());
    }
    let mut root: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let mut cache = HashMap::new();
    let mut count = 0;
    let mut resolve = |material: &mut Value| -> Result<(), String> {
        count += 1;
        if count > 256 {
            return Err("image material count limit".into());
        }
        let v = &material["image"];
        if v.as_object()
            .is_some_and(|o| o.len() != 1 || !o.contains_key("uri"))
        {
            return Err("constant image URI object required".into());
        }
        let raw = v
            .as_str()
            .or_else(|| v["uri"].as_str())
            .ok_or("constant image URI required")?;
        if raw.starts_with("data:") {
            return Ok(());
        }
        let url = base.join(raw).map_err(|e| e.to_string())?;
        if !matches!(url.scheme(), "http" | "https" | "file")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("image URL scheme/credentials".into());
        }
        if !cache.contains_key(url.as_str()) {
            if cache.len() >= 16 {
                return Err("unique image resource limit".into());
            }
            let data = read(&url, 4 * 1024 * 1024)?;
            if data.len() > 4 * 1024 * 1024 {
                return Err("image input limit".into());
            }
            let mime = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
                "png"
            } else if data.starts_with(b"\xff\xd8\xff") {
                "jpeg"
            } else {
                return Err("PNG/JPEG image required".into());
            };
            cache.insert(
                url.as_str().to_owned(),
                format!(
                    "data:image/{mime};base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(data)
                ),
            );
        }
        material["image"] = Value::String(cache[url.as_str()].clone());
        Ok(())
    };
    if let Some(ps) = root.get_mut("primitives").and_then(Value::as_array_mut) {
        for p in ps {
            if p["appearance"]["material"]["type"] == "Image" {
                resolve(&mut p["appearance"]["material"])?;
            }
        }
        let data = serde_json::to_vec(&root).map_err(|e| e.to_string())?;
        if data.len() > 8 * 1024 * 1024 {
            return Err("hydrated input byte limit".into());
        }
        return Ok(data);
    }
    let items = root
        .as_array_mut()
        .ok_or("entity/CZML array or primitives required")?;
    for item in items {
        if item["shape"]["type"] == "billboard" {
            resolve(&mut item["shape"])?;
        }
        if let Some(b) = item.get_mut("billboard") {
            resolve(b)?;
        }
        if item.get("shape").is_some() && item["material"]["type"] == "image" {
            resolve(&mut item["material"])?;
        }
        for key in [
            "box",
            "ellipsoid",
            "cylinder",
            "plane",
            "ellipse",
            "rectangle",
            "polygon",
            "wall",
        ] {
            if let Some(m) = item
                .get_mut(key)
                .and_then(|g| g.get_mut("material"))
                .and_then(|m| m.get_mut("image"))
            {
                resolve(m)?;
            }
        }
    }
    let out = serde_json::to_vec(&root).map_err(|e| e.to_string())?;
    if out.len() > 8 * 1024 * 1024 {
        return Err("hydrated input limit".into());
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_images_dedup_and_metadata_isolation() {
        let data = serde_json::json!([{"id":"one","shape":{},"material":{"type":"image","image":"logo.png"},"properties":{"image":"no-fetch.png"}},{"id":"two","rectangle":{"material":{"image":{"image":{"uri":"logo.png"}}}}}]);
        let mut reads = 0;
        let out = hydrate(
            &serde_json::to_vec(&data).unwrap(),
            &Url::parse("file:///demo/data/entities.json").unwrap(),
            |u, n| {
                reads += 1;
                assert_eq!(u.as_str(), "file:///demo/data/logo.png");
                assert_eq!(n, 4 * 1024 * 1024);
                Ok(b"\x89PNG\r\n\x1a\n".to_vec())
            },
        )
        .unwrap();
        assert_eq!(reads, 1);
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert!(v[0]["material"]["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert_eq!(v[0]["properties"]["image"], "no-fetch.png");
    }
    #[test]
    fn invalid_resource_failure() {
        let v = serde_json::json!([{"shape":{},"material":{"type":"image","image":"javascript:alert(1)"}}]);
        assert!(hydrate(
            &serde_json::to_vec(&v).unwrap(),
            &Url::parse("file:///demo/e.json").unwrap(),
            |_, _| panic!("invalid URL must not fetch")
        )
        .is_err());
    }
}
