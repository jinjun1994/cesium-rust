//! Optional native GPU smoke for decoded sample contents, isolated from the tileset tree.
use cesium_formats::decode_content;
use cesium_renderer::{device, instance, Renderer};
use cesium_viewer::{limits, Scene};
use std::{io::Read, sync::mpsc, time::Duration};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("sample-render-check <manifest.json>")?;
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(path)?)?;
    pollster::block_on(async {
        let instance = instance();
        let (_, device, queue) = device(&instance, None).await?;
        let mut renderer =
            Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 512, 384)?;
        for row in rows {
            let result = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
                let mut bytes = Vec::new();
                let limits = limits();
                std::fs::File::open(row["path"].as_str().ok_or("path")?)?
                    .take(limits.max_input_bytes as u64 + 1)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > limits.max_input_bytes {
                    return Err("input byte limit".into());
                }
                let content = decode_content(&bytes, &limits)?;
                let primitive_count = content.primitives.len();
                let mut scene = Scene::new(&mut renderer, None)?;
                scene.add_static(&mut renderer, content)?;
                let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("sample preview"),
                    size: wgpu::Extent3d {
                        width: 512,
                        height: 384,
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
                scene.render(&mut renderer, &view, None)?;
                let buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("sample readback"),
                    size: 512 * 384 * 4,
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
                            bytes_per_row: Some(2048),
                            rows_per_image: Some(384),
                        },
                    },
                    wgpu::Extent3d {
                        width: 512,
                        height: 384,
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
                let mapped = buffer.slice(..).get_mapped_range()?;
                let background = &mapped[..4];
                let changed = mapped
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|p| p.as_slice() != background)
                    .count();
                image::save_buffer(
                    row["output"].as_str().ok_or("output")?,
                    &mapped,
                    512,
                    384,
                    image::ColorType::Rgba8,
                )?;
                drop(mapped);
                buffer.unmap();
                // Check errors delivered during GPU completion before reporting success.
                scene.render(&mut renderer, &view, None)?;
                Ok(
                    serde_json::json!({"stage":"isolated-content-gpu-readback","primitives":primitive_count,"pixelsDifferentFromCorner":changed,"preview":row["preview"],"visiblePixelsObserved":changed>0}),
                )
            })();
            println!(
                "{}",
                match result {
                    Ok(result) =>
                        serde_json::json!({"id":row["id"],"status":"passed","result":result}),
                    Err(error) =>
                        serde_json::json!({"id":row["id"],"status":"failed","error":error.to_string()}),
                }
            );
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
