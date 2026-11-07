use std::collections::BTreeMap;
use cesium_renderer::{
    ShaderCache, ShaderContextInfo, ShaderProgramOptions,
};

#[test]
fn test_adds_and_removes() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());

    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs,
    });

    assert_eq!(sp.borrow().reference_count(), 1);
    assert_eq!(cache.number_of_shaders(), 1);

    cache.release_shader_program(&sp);
    assert!(!sp.borrow().is_destroyed());
    assert_eq!(cache.number_of_shaders(), 1);

    cache.destroy_released_shader_programs();
    assert!(sp.borrow().is_destroyed());
    assert_eq!(cache.number_of_shaders(), 0);

    cache.destroy();
}

#[test]
fn test_adds_and_removes_2() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs,
    });
    assert_eq!(sp.borrow().reference_count(), 1);

    sp.borrow_mut().destroy();
    assert!(!sp.borrow().is_destroyed());

    cache.destroy_released_shader_programs();
    assert!(sp.borrow().is_destroyed());

    cache.destroy();
}

#[test]
fn test_has_a_cache_hit() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs.clone(),
    });
    let sp2 = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs,
    });

    assert!(std::rc::Rc::ptr_eq(&sp, &sp2));
    assert_eq!(sp.borrow().reference_count(), 2);
    assert_eq!(cache.number_of_shaders(), 1);

    sp.borrow_mut().destroy();
    sp2.borrow_mut().destroy();
    cache.destroy_released_shader_programs();

    assert!(sp.borrow().is_destroyed());
    assert_eq!(cache.number_of_shaders(), 0);

    cache.destroy();
}

#[test]
fn test_cache_handles_unordered_attribute_locations() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs1 = BTreeMap::new();
    attrs1.insert("position".to_string(), 0);
    attrs1.insert("normal".to_string(), 1);
    attrs1.insert("color".to_string(), 2);

    let mut attrs2 = BTreeMap::new();
    attrs2.insert("color".to_string(), 2);
    attrs2.insert("position".to_string(), 0);
    attrs2.insert("normal".to_string(), 1);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp1 = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs1,
    });
    let sp2 = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs2,
    });

    assert!(std::rc::Rc::ptr_eq(&sp1, &sp2));
    assert_eq!(sp1.borrow().reference_count(), 2);
    assert_eq!(cache.number_of_shaders(), 1);

    sp1.borrow_mut().destroy();
    sp2.borrow_mut().destroy();
    cache.destroy_released_shader_programs();

    assert!(sp1.borrow().is_destroyed());
    assert_eq!(cache.number_of_shaders(), 0);

    cache.destroy();
}

#[test]
fn test_replaces_a_shader_program() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let fs2 = "void main() { out_FragColor = vec4(0.5); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.replace_shader_program(
        None,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs.into(),
            attribute_locations: attrs.clone(),
        },
    );

    let sp2 = cache.replace_shader_program(
        Some(&sp),
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs2.into(),
            attribute_locations: attrs,
        },
    );

    assert_eq!(sp.borrow().reference_count(), 0);
    assert_eq!(sp2.borrow().reference_count(), 1);

    cache.destroy();
}

#[test]
fn test_avoids_thrashing() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs.clone(),
    });
    sp.borrow_mut().destroy();

    let sp2 = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs,
    }); // still cache hit

    cache.destroy_released_shader_programs(); // does not destroy
    assert!(!sp.borrow().is_destroyed());
    assert!(!sp2.borrow().is_destroyed());

    sp2.borrow_mut().destroy();
    cache.destroy_released_shader_programs(); // now destroys

    assert!(sp.borrow().is_destroyed());
    assert!(sp2.borrow().is_destroyed());

    cache.destroy();
}

#[test]
fn test_create_and_get_derived_shader_program() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs.clone(),
    });

    let keyword = "derived";
    let sp_derived = cache.get_derived_shader_program(&sp, keyword);
    assert!(sp_derived.is_none());

    let fs_derived = "void main() { out_FragColor = vec4(vec3(1.0), 0.5); }";
    let sp_derived = cache.create_derived_shader_program(
        &sp,
        keyword,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs_derived.into(),
            attribute_locations: attrs,
        },
    );

    let fetched = cache.get_derived_shader_program(&sp, keyword);
    assert!(fetched.is_some());
    assert!(std::rc::Rc::ptr_eq(&sp_derived, &fetched.unwrap()));

    cache.destroy();
}

#[test]
fn test_replaces_derived_shader_program() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs.clone(),
    });

    let keyword = "derived";
    let fs_derived = "void main() { out_FragColor = vec4(vec3(1.0), 0.5); }";
    let sp_derived = cache.replace_derived_shader_program(
        &sp,
        keyword,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs_derived.into(),
            attribute_locations: attrs.clone(),
        },
    );

    assert_eq!(sp.borrow().derived_keywords().len(), 1);

    let fs_derived2 = "void main() { out_FragColor = vec4(vec3(0.5), 0.5); }";
    let sp_derived2 = cache.replace_derived_shader_program(
        &sp,
        keyword,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs_derived2.into(),
            attribute_locations: attrs,
        },
    );

    assert!(sp_derived.borrow().is_destroyed());
    assert!(!sp_derived2.borrow().is_destroyed());
    assert_eq!(sp.borrow().derived_keywords().len(), 1);

    cache.destroy();
}

#[test]
fn test_destroying_a_shader_program_destroys_its_derived_shaders() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs.clone(),
    });

    let keyword = "derived";
    let fs_derived = "void main() { out_FragColor = vec4(vec3(1.0), 0.5); }";
    let sp_derived = cache.create_derived_shader_program(
        &sp,
        keyword,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs_derived.into(),
            attribute_locations: attrs,
        },
    );

    sp.borrow_mut().destroy();
    cache.destroy_released_shader_programs();

    assert!(sp.borrow().is_destroyed());
    assert!(sp_derived.borrow().is_destroyed());

    cache.destroy();
}

#[test]
fn test_cache_is_destroyed_and_not_destroyed() {
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 0);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    assert!(!cache.is_destroyed());

    let sp = cache.get_shader_program(ShaderProgramOptions {
        vertex_shader_source: vs.into(),
        fragment_shader_source: fs.into(),
        attribute_locations: attrs,
    });

    cache.destroy();
    assert!(sp.borrow().is_destroyed());
    assert!(cache.is_destroyed());
}
