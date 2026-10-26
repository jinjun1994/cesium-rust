//! Deterministic GPU render of the SAME default Scene used by desktop/Web viewers.
//! Waiting on the GPU is deliberate here: this is an export/verification utility.
use cesium_renderer::{attachment, device, instance, Renderer};
use cesium_viewer::Scene;
use std::{path::PathBuf, sync::mpsc, time::Duration};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "earth-preview.png".into()),
    );
    let imagery_dir = std::env::var("CESIUM_TEST_IMAGERY_DIR").ok();
    pollster::block_on(async {
        let instance = instance();
        let (adapter, device, queue) = device(&instance, None).await?;
        let (width, height) = (1280u32, 800u32);
        let mut renderer = Renderer::new(
            device,
            queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
        )?;
        let mut scene = Scene::new(&mut renderer, None)?;
        scene.select(width, height);
        if let Some(directory) = imagery_dir {
            use cesium_viewer::imagery::{Projection, Provider};
            let template = format!("file://{directory}/{{z}}/{{x}}/{{reverseY}}.jpg");
            scene.set_imagery(
                &mut renderer,
                Provider::template(
                    &template,
                    Projection::Geographic,
                    2,
                    "Natural Earth II · Cesium sample imagery",
                )?,
            )?;
            // Deterministic export utility: same selection/decoder/upload as async viewer, local cached tiles.
            for _ in 0..64 {
                scene.update_imagery(&mut renderer)?;
                let requests = scene.imagery.as_mut().unwrap().take_requests(4);
                if requests.is_empty() {
                    break;
                }
                for (tile, url) in requests {
                    let bytes = std::fs::read(url.to_file_path().map_err(|_| "test tile path")?)?;
                    let image = cesium_viewer::imagery::decode(&bytes)?;
                    scene
                        .imagery
                        .as_mut()
                        .unwrap()
                        .submit(&mut renderer, tile, image)?;
                }
            }
            scene.update_imagery(&mut renderer)?;
        }
        if let Ok(path) = std::env::var("CESIUM_TEST_GEOJSON") {
            let bytes = std::fs::read(path)?;
            let style = cesium_viewer::geojson::Style {
                point_radius: 100000.0,
                line_width: 50000.0,
                surface_offset: 10000.0,
                ..Default::default()
            };
            let data = cesium_viewer::geojson::decode(&bytes, &style, &Default::default())?;
            scene.add_geojson(&mut renderer, data)?;
        }
        let target = attachment(
            &renderer.device,
            width,
            height,
            renderer.format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        scene.render(
            &mut renderer,
            &target.create_view(&Default::default()),
            None,
        )?;
        let pitch = (width * 4).div_ceil(256) * 256;
        let buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Earth export readback"),
            size: pitch as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = renderer.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pitch),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        renderer.queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        renderer.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })?;
        rx.recv()?.map_err(|e| format!("map: {e}"))?;
        let view = buffer.slice(..).get_mapped_range()?;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for row in view.chunks_exact(pitch as usize) {
            pixels.extend_from_slice(&row[..(width * 4) as usize]);
        }
        drop(view);
        buffer.unmap();
        // Reject a blank frame or the old close-up triangle: a bounded, richly colored disk must exist.
        let background = &pixels[..3];
        let (mut left, mut right, mut top, mut bottom, mut changed) = (width, 0, height, 0, 0usize);
        let mut colors = std::collections::HashSet::new();
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                if (0..3).any(|k| pixels[i + k].abs_diff(background[k]) > 8) {
                    left = left.min(x);
                    right = right.max(x);
                    top = top.min(y);
                    bottom = bottom.max(y);
                    changed += 1;
                    colors.insert([pixels[i], pixels[i + 1], pixels[i + 2]]);
                }
            }
        }
        if changed < (width * height / 10) as usize
            || left == 0
            || top == 0
            || right >= width - 1
            || bottom >= height - 1
            || colors.len() < 1024
        {
            return Err(format!("Earth image validation failed: pixels={changed}, colors={}, bounds={left},{top}..{right},{bottom}",colors.len()).into());
        }
        if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        image::save_buffer(&output, &pixels, width, height, image::ColorType::Rgba8)?;
        println!("Earth rendered: {}x{}, adapter {:?}, {} colored pixels, {} colors, bounds ({left},{top})..({right},{bottom}); {}",width,height,adapter.get_info().backend,changed,colors.len(),scene.stats());
        println!("saved {}", output.display());
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
