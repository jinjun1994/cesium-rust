//! Pipeline keys and RenderPipeline caching.

use cesium_formats::Vertex;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct PipelineKey {
    pub pick: bool,
    pub blend: bool,
    pub line: bool,
    pub variant: u8,
    pub overlay: bool,
}

pub struct PipelineCache {
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
}

impl PipelineCache {
    pub fn build(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let mut pipelines = HashMap::new();
        for pick in [false, true] {
            for variant in 0..5 {
                for blend in [false, true] {
                    for line in [false, true] {
                        for overlay in [false, true] {
                            let key = PipelineKey {
                                pick,
                                variant,
                                blend,
                                line,
                                overlay,
                            };
                            let target = if pick {
                                wgpu::TextureFormat::R32Uint
                            } else {
                                color_format
                            };
                            let attributes = wgpu::vertex_attr_array![
                                0 => Float32x3,
                                1 => Float32x3,
                                2 => Float32x2,
                                3 => Uint32,
                                4 => Float32x2,
                                5 => Float32x4,
                                6 => Float32x4
                            ];
                            let pipeline =
                                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                    label: Some("cesium pipeline"),
                                    layout: Some(layout),
                                    vertex: wgpu::VertexState {
                                        module: shader,
                                        entry_point: Some("vs_main"),
                                        compilation_options: Default::default(),
                                        buffers: &[Some(wgpu::VertexBufferLayout {
                                            array_stride: std::mem::size_of::<Vertex>() as u64,
                                            step_mode: wgpu::VertexStepMode::Vertex,
                                            attributes: &attributes,
                                        })],
                                    },
                                    primitive: wgpu::PrimitiveState {
                                        topology: if line {
                                            wgpu::PrimitiveTopology::LineList
                                        } else {
                                            wgpu::PrimitiveTopology::TriangleList
                                        },
                                        cull_mode: if line || variant == 0 {
                                            None
                                        } else {
                                            Some(if variant >= 3 {
                                                wgpu::Face::Front
                                            } else {
                                                wgpu::Face::Back
                                            })
                                        },
                                        front_face: if variant == 2 || variant == 4 {
                                            wgpu::FrontFace::Cw
                                        } else {
                                            wgpu::FrontFace::Ccw
                                        },
                                        ..Default::default()
                                    },
                                    depth_stencil: Some(wgpu::DepthStencilState {
                                        format: wgpu::TextureFormat::Depth32Float,
                                        depth_write_enabled: Some(!overlay && (pick || !blend)),
                                        depth_compare: Some(if overlay {
                                            wgpu::CompareFunction::Always
                                        } else {
                                            wgpu::CompareFunction::GreaterEqual
                                        }),
                                        stencil: Default::default(),
                                        bias: Default::default(),
                                    }),
                                    multisample: Default::default(),
                                    fragment: Some(wgpu::FragmentState {
                                        module: shader,
                                        entry_point: Some(if pick {
                                            "fs_pick"
                                        } else {
                                            "fs_color"
                                        }),
                                        compilation_options: Default::default(),
                                        targets: &[Some(wgpu::ColorTargetState {
                                            format: target,
                                            blend: if blend && !pick {
                                                Some(wgpu::BlendState::ALPHA_BLENDING)
                                            } else {
                                                None
                                            },
                                            write_mask: wgpu::ColorWrites::ALL,
                                        })],
                                    }),
                                    multiview_mask: None,
                                    cache: None,
                                });
                            pipelines.insert(key, pipeline);
                        }
                    }
                }
            }
        }
        Self { pipelines }
    }

    #[inline]
    pub fn get(&self, key: &PipelineKey) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(key)
    }
}
