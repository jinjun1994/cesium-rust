use cesium_renderer::Renderer;
use cesium_viewer::Scene;
use std::{sync::mpsc, time::Duration};
pub fn pixels(
    renderer: &mut Renderer,
    scene: &Scene,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
/// Publish complete PNG artifacts so interrupted writes cannot truncate a previous preview.
pub fn save_png(
    path: impl AsRef<std::path::Path>,
    bytes: &[u8],
    width: u32,
    height: u32,
    color: image::ColorType,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::ImageEncoder;
    use std::io::Write;
    let path = path.as_ref();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png).write_image(
        bytes,
        width,
        height,
        color.into(),
    )?;
    let temp = path.with_extension("png.tmp");
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(&png)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temp, path)?;
    Ok(())
}
