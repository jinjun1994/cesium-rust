//! Real public terrain HTTP streaming plus offscreen GPU/readback verification.
use cesium_renderer::{device, instance, Renderer};
use cesium_viewer::{
    terrain::{self, Provider},
    Scene,
};
use std::{io::Write, path::Path, sync::mpsc, time::Duration};
fn pixels(renderer: &mut Renderer, scene: &Scene) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("terrain check"),
        size: wgpu::Extent3d {
            width: 768,
            height: 512,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    scene.render(renderer, &view, None)?;
    let buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("terrain readback"),
        size: 768 * 512 * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = renderer.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(3072),
                rows_per_image: Some(512),
            },
        },
        wgpu::Extent3d {
            width: 768,
            height: 512,
            depth_or_array_layers: 1,
        },
    );
    renderer.queue.submit([encoder.finish()]);
    let (tx, rx) = mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    renderer.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(30)),
    })?;
    rx.recv_timeout(Duration::from_secs(30))??;
    let map = buffer.slice(..).get_mapped_range()?;
    let out = map.to_vec();
    drop(map);
    buffer.unmap();
    scene.render(renderer, &view, None)?;
    Ok(out)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let offline = args.iter().any(|a| a == "--cached-only");
    let stages = args
        .iter()
        .find_map(|a| a.strip_prefix("--stages="))
        .map(str::parse)
        .transpose()?
        .unwrap_or(24usize);
    if !(1..=64).contains(&stages) {
        return Err("stages 1..64".into());
    }
    let cache = Path::new(".terrain-cache");
    std::fs::create_dir_all(cache)?;
    std::fs::create_dir_all("docs")?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    pollster::block_on(async {
        let inst = instance();
        let (_, device, queue) = device(&inst, None).await?;
        let mut renderer =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 768, 512)?;
        let mut scene = Scene::new(&mut renderer, None)?;
        scene.set_terrain(&mut renderer, Provider::public_terrain())?;
        scene.look_at_terrain(86.92528, 27.9881, 20000.0)?;
        let mut fetched = 0;
        let mut failures = Vec::new();
        for stage in 0..stages {
            scene.update_terrain(&mut renderer)?;
            let layer = scene.terrain.as_mut().unwrap();
            let requests = layer.take_requests(4);
            if requests.is_empty() {
                break;
            }
            let results: Vec<_> = std::thread::scope(|scope| {
                let jobs: Vec<_> = requests
                    .into_iter()
                    .map(|(tile, url)| {
                        let http = client.clone();
                        let accept = layer.provider.accept().to_owned();
                        scope.spawn(move || {
                            let path = cache.join(tile.key().replace('/', "-") + ".png");
                            let result = (|| -> Result<Vec<u8>, String> {
                                if let Ok(bytes) = std::fs::read(&path) {
                                    if bytes.len() <= terrain::MAX_TILE_BYTES {
                                        return Ok(bytes);
                                    }
                                }
                                if offline {
                                    return Err("tile not cached".into());
                                }
                                let bytes = cesium_viewer::native::fetch_terrain(
                                    &url,
                                    &http,
                                    terrain::MAX_TILE_BYTES,
                                    &accept,
                                    None,
                                )
                                .map_err(|_| "terrain HTTP failed")?;
                                let mut f = std::fs::File::create(path)
                                    .map_err(|_| "terrain cache write")?;
                                f.write_all(&bytes).map_err(|_| "terrain cache write")?;
                                Ok(bytes)
                            })();
                            (tile, result)
                        })
                    })
                    .collect();
                jobs.into_iter().map(|job| job.join().unwrap()).collect()
            });
            for (tile, result) in results {
                match result.and_then(|b| terrain::decode(&b, terrain::Format::Terrarium)) {
                    Ok(data) => match layer.submit(&mut renderer, tile, data, None) {
                        Ok(()) => fetched += 1,
                        Err(e) => {
                            layer.fail(tile);
                            failures.push(format!("{}: {e}", tile.key()));
                        }
                    },
                    Err(e) => {
                        layer.fail(tile);
                        failures.push(format!("{}: {e}", tile.key()));
                    }
                }
            }
            println!("stage {stage}: {}", layer.stats());
            std::io::stdout().flush()?;
        }
        scene.update_terrain(&mut renderer)?;
        let before = pixels(&mut renderer, &scene)?;
        let height = scene
            .terrain
            .as_ref()
            .and_then(|t| t.height_at(86.92528f64.to_radians(), 27.9881f64.to_radians()));
        let stats = scene.stats();
        let credits = scene.terrain.as_ref().unwrap().provider.credits.clone();
        image::save_buffer(
            "docs/terrain-preview.png",
            &before,
            768,
            512,
            image::ColorType::Rgba8,
        )?;
        scene.clear_terrain();
        let after = pixels(&mut renderer, &scene)?;
        image::save_buffer(
            "docs/terrain-ellipsoid-preview.png",
            &after,
            768,
            512,
            image::ColorType::Rgba8,
        )?;
        let differences = before
            .as_chunks::<4>()
            .0
            .iter()
            .zip(after.as_chunks::<4>().0)
            .filter(|(a, b)| a != b)
            .count();
        let record = serde_json::json!({"source":"https://registry.opendata.aws/terrain-tiles/","location":{"longitude":86.92528,"latitude":27.9881},"decodedTiles":fetched,"cachedOnly":offline,"height":height,"pixelsDifferentFromEllipsoid":differences,"stats":stats,"credits":credits,"failures":failures,"scope":"Native public Terrarium HTTP streaming and GPU readback. Not a Cesium ion live-service test or browser rendering test."});
        std::fs::write(
            "docs/terrain-validation.json",
            serde_json::to_vec_pretty(&record)?,
        )?;
        println!("{record}");
        if fetched == 0 || differences < 100 {
            return Err("no terrain visible in readback".into());
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
