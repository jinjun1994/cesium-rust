//! ShadowMapShader matching CesiumJS `Scene/ShadowMapShader.js` (Apache-2.0).

use crate::shader_source::ShaderSource;

/// Helper methods for generating shadow casting and shadow receiving shaders.
#[derive(Debug, Clone, Copy)]
pub struct ShadowMapShader;

impl ShadowMapShader {
    /// Returns true if the fragment shader clips fragments via a `ClippingPlaneCollection`
    /// or `ClippingPolygonCollection`, in which case the shadow cast pass must run
    /// the original main so that clipped fragments are discarded and do not cast shadows.
    pub fn has_clipping_for_shadow_cast(fragment_shader_source: &ShaderSource) -> bool {
        fragment_shader_source.defines.iter().any(|define| {
            define == "HAS_CLIPPING_PLANES" || define == "ENABLE_CLIPPING_POLYGONS"
        })
    }

    /// Generates a cache keyword for shadow casting shaders.
    pub fn get_shadow_cast_shader_keyword(
        is_point_light: bool,
        is_terrain: bool,
        uses_depth_texture: bool,
        is_opaque: bool,
        has_clipping: bool,
    ) -> String {
        format!(
            "castShadow {is_point_light} {is_terrain} {uses_depth_texture} {is_opaque} {has_clipping}"
        )
    }

    /// Creates a shadow cast vertex shader from an existing vertex shader.
    pub fn create_shadow_cast_vertex_shader(
        vs: &ShaderSource,
        is_point_light: bool,
        is_terrain: bool,
    ) -> ShaderSource {
        let mut defines = vs.defines.clone();
        let mut sources = vs.sources.clone();

        defines.push("SHADOW_MAP".to_string());

        if is_terrain {
            defines.push("GENERATE_POSITION".to_string());
        }

        let position_varying_name = vs.find_position_varying();
        let has_position_varying = position_varying_name.is_some();

        if is_point_light && !has_position_varying {
            for source in &mut sources {
                *source = ShaderSource::replace_main(source, "czm_shadow_cast_main");
            }

            let shadow_vs = "out vec3 v_positionEC; \n\
void main() \n\
{ \n\
    czm_shadow_cast_main(); \n\
    v_positionEC = (czm_inverseProjection * gl_Position).xyz; \n\
}\n";
            sources.push(shadow_vs.to_string());
        }

        ShaderSource {
            defines,
            sources,
            pick_color_qualifier: vs.pick_color_qualifier.clone(),
            include_built_ins: vs.include_built_ins,
        }
    }

    /// Creates a shadow cast fragment shader from an existing fragment shader.
    pub fn create_shadow_cast_fragment_shader(
        fs: &ShaderSource,
        is_point_light: bool,
        uses_depth_texture: bool,
        opaque: bool,
        has_clipping: bool,
    ) -> ShaderSource {
        let mut defines = fs.defines.clone();
        let mut sources = fs.sources.clone();

        defines.push("SHADOW_MAP".to_string());

        let position_varying_name = fs
            .find_position_varying()
            .unwrap_or_else(|| "v_positionEC".to_string());
        let has_position_varying = fs.find_position_varying().is_some();

        for source in &mut sources {
            *source = ShaderSource::replace_main(source, "czm_shadow_cast_main");
        }

        let mut fs_source = String::new();

        if is_point_light {
            if !has_position_varying {
                fs_source.push_str("in vec3 v_positionEC; \n");
            }
            fs_source.push_str("uniform vec4 shadowMap_lightPositionEC; \n");
        }

        if opaque && has_clipping {
            fs_source.push_str("void main() \n{ \n    czm_shadow_cast_main(); \n");
        } else if opaque {
            fs_source.push_str("void main() \n{ \n");
        } else {
            fs_source.push_str(
                "void main() \n\
{ \n\
    czm_shadow_cast_main(); \n\
    if (out_FragColor.a == 0.0) \n\
    { \n\
       discard; \n\
    } \n",
            );
        }

        if is_point_light {
            fs_source.push_str(&format!(
                "    float distance = length({position_varying_name}); \n\
    if (distance >= shadowMap_lightPositionEC.w) \n\
    {{ \n\
        discard; \n\
    }} \n\
    distance /= shadowMap_lightPositionEC.w; // radius \n\
    out_FragColor = czm_packDepth(distance); \n"
            ));
        } else if uses_depth_texture {
            fs_source.push_str("    out_FragColor = vec4(1.0); \n");
        } else {
            fs_source.push_str("    out_FragColor = czm_packDepth(gl_FragCoord.z); \n");
        }

        fs_source.push_str("}\n");
        sources.push(fs_source);

        ShaderSource {
            defines,
            sources,
            pick_color_qualifier: fs.pick_color_qualifier.clone(),
            include_built_ins: fs.include_built_ins,
        }
    }

    /// Generates a cache keyword for shadow receiving shaders.
    #[allow(clippy::too_many_arguments)]
    pub fn get_shadow_receive_shader_keyword(
        uses_depth_texture: bool,
        polygon_offset_supported: bool,
        is_point_light: bool,
        is_spot_light: bool,
        has_cascades: bool,
        debug_cascade_colors: bool,
        soft_shadows: bool,
        cast_shadows: bool,
        is_terrain: bool,
        has_terrain_normal: bool,
    ) -> String {
        format!(
            "receiveShadow {uses_depth_texture}{polygon_offset_supported}{is_point_light}{is_spot_light}{has_cascades}{debug_cascade_colors}{soft_shadows}{cast_shadows}{is_terrain}{has_terrain_normal}"
        )
    }
}
