use cesium_formats::{decode_content, parse_b3dm, parse_glb, DecodeLimits};
use cesium_tiles::Tileset;
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    if command == "batch" {
        batch()?;
        return Ok(());
    }
    if command == "capabilities" {
        println!("{}", include_str!("../../../docs/capabilities.json"));
        return Ok(());
    }
    if !["validate", "inspect"].contains(&command.as_str()) {
        println!("asset-inspector validate|inspect <file> | capabilities");
        return Ok(());
    }
    let path = args.next().ok_or("missing asset path")?;
    let limits = DecodeLimits {
        max_decoded_bytes: 64 * 1024 * 1024,
        max_nodes: 10_000,
        ..Default::default()
    };
    let file = std::fs::File::open(&path)?;
    let mut data = Vec::new();
    file.take(limits.max_input_bytes as u64 + 1)
        .read_to_end(&mut data)?;
    if data.len() > limits.max_input_bytes {
        return Err("input size limit".into());
    }
    if data.starts_with(b"b3dm") {
        let b = parse_b3dm(&data, &limits)?;
        println!(
            "b3dm: batches {}, RTC {:?}, batch table {}",
            b.batch_length, b.rtc_center, b.batch_table
        );
        let c = decode_content(&data, &limits)?;
        println!(
            "decoded: {} primitives, {} bytes",
            c.primitives.len(),
            c.decoded_bytes
        );
    } else if data.starts_with(b"glTF") {
        let glb = parse_glb(&data, &limits)?;
        if command == "inspect" {
            println!("{}", serde_json::to_string_pretty(&glb.json)?);
        }
        let c = decode_content(&data, &limits)?;
        println!(
            "GLB: {} primitives, {} decoded bytes",
            c.primitives.len(),
            c.decoded_bytes
        );
    } else {
        let url = url::Url::from_file_path(std::fs::canonicalize(path)?).map_err(|_| "path URL")?;
        let t = Tileset::parse(&data, url, &limits)?;
        println!("tileset: {} explicit nodes", t.tiles.len());
        if command == "inspect" {
            for (i, n) in t.tiles.iter().enumerate() {
                println!(
                    "tile {i}: {:?}, error {}, children {}, content {}",
                    n.refine,
                    n.geometric_error,
                    n.children.len(),
                    n.uri.is_some()
                );
            }
        }
    }
    println!("valid for the declared implementation subset");
    Ok(())
}

/// JSON-lines protocol runs production parsers with separate failures per input.
fn batch() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::BufRead;
    let limits = DecodeLimits {
        max_decoded_bytes: 64 * 1024 * 1024,
        max_nodes: 10_000,
        ..Default::default()
    };
    for line in std::io::stdin().lock().lines() {
        let row: serde_json::Value = serde_json::from_str(&line?)?;
        let result = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let mut data = Vec::new();
            std::fs::File::open(row["path"].as_str().ok_or("missing path")?)?
                .take(limits.max_input_bytes as u64 + 1)
                .read_to_end(&mut data)?;
            if data.len() > limits.max_input_bytes {
                return Err("input size limit".into());
            }
            if data.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{') {
                let source = url::Url::parse(row["url"].as_str().ok_or("missing source URL")?)?;
                let tileset = Tileset::parse(&data, source, &limits)?;
                Ok(
                    serde_json::json!({"stage":"tileset-parse","nodes":tileset.tiles.len(),
                    "contents":tileset.tiles.iter().filter_map(|t| t.uri.as_ref()).map(|u| u.as_str()).collect::<Vec<_>>()}),
                )
            } else {
                let content = decode_content(&data, &limits)?;
                let glb = if data.starts_with(b"b3dm") {
                    parse_b3dm(&data, &limits)?.glb
                } else {
                    &data
                };
                let parsed = parse_glb(glb, &limits)?;
                Ok(
                    serde_json::json!({"stage":"content-decode","primitives":content.primitives.len(),"decodedBytes":content.decoded_bytes,"declaredExtensions":parsed.json.get("extensionsUsed")}),
                )
            }
        })();
        println!(
            "{}",
            match result {
                Ok(value) => serde_json::json!({"id":row["id"],"status":"passed","result":value}),
                Err(error) =>
                    serde_json::json!({"id":row["id"],"status":"failed","error":error.to_string()}),
            }
        );
    }
    Ok(())
}
