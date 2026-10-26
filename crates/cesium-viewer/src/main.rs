#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use cesium_viewer::native::{load, load_geojson, run, source_url, Startup};
    let mut source = None;
    let mut smoke = None;
    let mut demo_tile = false;
    let mut overlays = Vec::new();
    for arg in std::env::args().skip(1) {
        if let Some(path) = arg.strip_prefix("--geojson=") {
            if overlays.len() >= 32 {
                return Err("at most 32 GeoJSON layers".into());
            }
            overlays.push(load_geojson(source_url(path)?)?);
        } else if let Some(n) = arg.strip_prefix("--smoke-frames=") {
            smoke = Some(n.parse()?);
        } else if arg == "--demo-tile" {
            demo_tile = true;
        } else if arg == "--help" {
            println!("desktop-viewer [tileset.json|model.glb|tile.b3dm|data.geojson|URL] [--geojson=URL_OR_PATH] [--smoke-frames=N]\nNo input: progressive WGS84 imagery and public Terrarium terrain. CESIUM_TERRAIN=0: disable terrain; CESIUM_TERRAIN_URL: layer.json or Terrarium XYZ template; CESIUM_VIEW_LOCATION: lon,lat,height. CESIUM_ION_TOKEN: Cesium default aerial imagery; no token: Natural Earth II (levels 0..2). CESIUM_OFFLINE=1: bundled NASA base. --demo-tile: original parser/picking fixture. CESIUM_CAMERA: local JSON array of camera commands (<=16 commands / 16 KiB). C: cancel flight; U: stop tracking. Left drag: orbit; wheel: zoom; click: pick; Esc: exit.");
            return Ok(());
        } else if source.replace(arg).is_some() {
            return Err("only one input asset is accepted".into());
        }
    }
    if demo_tile && source.is_some() {
        return Err("--demo-tile and asset input are mutually exclusive".into());
    }
    let startup = match source {
        Some(s) => load(source_url(&s)?)?,
        None => {
            if demo_tile {
                Startup::Demo
            } else {
                Startup::Globe
            }
        }
    };
    let startup = if overlays.is_empty() {
        startup
    } else {
        Startup::WithGeoJson(Box::new(startup), overlays)
    };
    run(startup, smoke)
}
#[cfg(target_arch = "wasm32")]
fn main() {}
