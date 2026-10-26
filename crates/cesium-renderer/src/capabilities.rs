//! Hardware capabilities negotiation and adapter limit inspection.

/// Probed hardware and driver capabilities for the current adapter and surface.
#[derive(Clone, Debug)]
pub struct RendererCapabilities {
    pub supports_timestamp_query: bool,
    pub max_texture_dimension_2d: u32,
    pub max_buffer_size: u64,
    pub min_uniform_buffer_offset_alignment: u32,
    pub max_uniform_buffer_binding_size: u64,
}

impl RendererCapabilities {
    pub fn probe(adapter: &wgpu::Adapter) -> Self {
        let features = adapter.features();
        let limits = adapter.limits();

        Self {
            supports_timestamp_query: features.contains(wgpu::Features::TIMESTAMP_QUERY),
            max_texture_dimension_2d: limits.max_texture_dimension_2d,
            max_buffer_size: limits.max_buffer_size,
            min_uniform_buffer_offset_alignment: limits.min_uniform_buffer_offset_alignment,
            max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size,
        }
    }
}
