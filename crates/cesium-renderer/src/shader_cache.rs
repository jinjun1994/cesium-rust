//! ShaderCache & ShaderProgram matching CesiumJS `Renderer/ShaderCache.js` & `Renderer/ShaderProgram.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::{Rc, Weak};

use crate::shader_source::{ShaderContextInfo, ShaderSource, ShaderSourceOptions};

/// Options for acquiring or compiling a shader program.
#[derive(Debug, Clone, Default)]
pub struct ShaderProgramOptions {
    pub vertex_shader_source: ShaderSourceInput,
    pub fragment_shader_source: ShaderSourceInput,
    pub attribute_locations: BTreeMap<String, u32>,
}

#[derive(Debug, Clone)]
pub enum ShaderSourceInput {
    Raw(String),
    Source(ShaderSource),
}

impl Default for ShaderSourceInput {
    fn default() -> Self {
        Self::Raw(String::new())
    }
}

impl From<&str> for ShaderSourceInput {
    fn from(s: &str) -> Self {
        Self::Raw(s.to_string())
    }
}

impl From<String> for ShaderSourceInput {
    fn from(s: String) -> Self {
        Self::Raw(s)
    }
}

impl From<ShaderSource> for ShaderSourceInput {
    fn from(s: ShaderSource) -> Self {
        Self::Source(s)
    }
}

impl ShaderSourceInput {
    pub fn to_shader_source(&self) -> ShaderSource {
        match self {
            Self::Raw(s) => ShaderSource::new(ShaderSourceOptions {
                sources: if s.is_empty() { Vec::new() } else { vec![s.clone()] },
                ..Default::default()
            })
            .unwrap(),
            Self::Source(s) => s.clone(),
        }
    }
}

/// A compiled and linked GLSL shader program abstraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexAttributeDesc {
    pub name: String,
    pub index: u32,
    pub glsl_type: String,
}

pub struct ShaderProgram {
    pub vertex_shader_source: ShaderSource,
    pub fragment_shader_source: ShaderSource,
    pub vertex_shader_text: String,
    pub fragment_shader_text: String,
    pub attribute_locations: BTreeMap<String, u32>,
    pub vertex_attributes: BTreeMap<String, VertexAttributeDesc>,
    pub(crate) cached_entry: Option<Weak<RefCell<CachedShaderEntry>>>,
    pub(crate) is_destroyed: bool,
}

impl std::fmt::Debug for ShaderProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderProgram")
            .field("vertex_shader_text", &self.vertex_shader_text)
            .field("fragment_shader_text", &self.fragment_shader_text)
            .field("attribute_locations", &self.attribute_locations)
            .field("vertex_attributes", &self.vertex_attributes)
            .field("is_destroyed", &self.is_destroyed)
            .finish()
    }
}

impl ShaderProgram {
    pub fn new(
        vertex_shader_source: ShaderSource,
        vertex_shader_text: String,
        fragment_shader_source: ShaderSource,
        fragment_shader_text: String,
        attribute_locations: BTreeMap<String, u32>,
    ) -> Self {
        let vertex_attributes = parse_vertex_attributes(&vertex_shader_text, &attribute_locations);
        Self {
            vertex_shader_source,
            fragment_shader_source,
            vertex_shader_text,
            fragment_shader_text,
            attribute_locations,
            vertex_attributes,
            cached_entry: None,
            is_destroyed: false,
        }
    }

    pub fn from_cache(
        cache: &ShaderCache,
        options: ShaderProgramOptions,
    ) -> Rc<RefCell<Self>> {
        cache.get_shader_program(options)
    }

    pub fn number_of_vertex_attributes(&self) -> usize {
        self.vertex_attributes.len()
    }

    pub fn reference_count(&self) -> usize {
        self.cached_entry
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|e| e.borrow().count)
            .unwrap_or(0)
    }

    pub fn derived_keywords(&self) -> Vec<String> {
        self.cached_entry
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|e| e.borrow().derived_keywords.clone())
            .unwrap_or_default()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if let Some(ref weak) = self.cached_entry {
            if let Some(entry) = weak.upgrade() {
                let cache_weak = entry.borrow().cache.clone();
                if let Some(cache_rc) = cache_weak.and_then(|c| c.upgrade()) {
                    cache_rc.borrow_mut().release_entry(entry.clone());
                }
            }
        }
    }

    pub(crate) fn final_destroy(&mut self) {
        self.is_destroyed = true;
    }
}

pub struct CachedShaderEntry {
    pub keyword: String,
    pub shader_program: Rc<RefCell<ShaderProgram>>,
    pub count: usize,
    pub derived_keywords: Vec<String>,
    pub(crate) cache: Option<Weak<RefCell<ShaderCacheInner>>>,
}

pub struct ShaderCacheInner {
    pub context: ShaderContextInfo,
    pub shaders: HashMap<String, Rc<RefCell<CachedShaderEntry>>>,
    pub shaders_to_release: HashMap<String, Rc<RefCell<CachedShaderEntry>>>,
    pub number_of_shaders: usize,
    pub is_destroyed: bool,
}

impl ShaderCacheInner {
    pub fn release_entry(&mut self, entry: Rc<RefCell<CachedShaderEntry>>) {
        let mut e = entry.borrow_mut();
        if e.count > 0 {
            e.count -= 1;
        }
        if e.count == 0 {
            self.shaders_to_release
                .insert(e.keyword.clone(), entry.clone());
        }
    }

    fn destroy_shader_recursive(&mut self, entry: Rc<RefCell<CachedShaderEntry>>) {
        let derived_keywords = entry.borrow().derived_keywords.clone();
        let base_keyword = entry.borrow().keyword.clone();

        for kw in derived_keywords {
            let derived_key = format!("{kw}{base_keyword}");
            if let Some(derived_entry) = self.shaders.remove(&derived_key) {
                self.destroy_shader_recursive(derived_entry);
            }
        }

        self.shaders.remove(&base_keyword);
        entry.borrow_mut().shader_program.borrow_mut().final_destroy();
    }
}

/// Thread-safe or shared reference to ShaderCache.
#[derive(Clone)]
pub struct ShaderCache {
    inner: Rc<RefCell<ShaderCacheInner>>,
}

impl ShaderCache {
    pub fn new(context: ShaderContextInfo) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ShaderCacheInner {
                context,
                shaders: HashMap::new(),
                shaders_to_release: HashMap::new(),
                number_of_shaders: 0,
                is_destroyed: false,
            })),
        }
    }

    pub fn number_of_shaders(&self) -> usize {
        self.inner.borrow().number_of_shaders
    }

    pub fn is_destroyed(&self) -> bool {
        self.inner.borrow().is_destroyed
    }

    fn build_keyword(
        vs: &ShaderSource,
        fs: &ShaderSource,
        attrs: &BTreeMap<String, u32>,
    ) -> String {
        let vs_key = vs.get_cache_key();
        let fs_key = fs.get_cache_key();
        let attrs_key = attrs
            .iter()
            .map(|(k, v)| format!("\"{k}\":{v}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("{vs_key}:{fs_key}:{{{attrs_key}}}")
    }

    pub fn get_shader_program(&self, options: ShaderProgramOptions) -> Rc<RefCell<ShaderProgram>> {
        let vs = options.vertex_shader_source.to_shader_source();
        let fs = options.fragment_shader_source.to_shader_source();
        let attrs = options.attribute_locations;

        let keyword = Self::build_keyword(&vs, &fs, &attrs);

        let mut inner = self.inner.borrow_mut();

        if let Some(cached) = inner.shaders.get(&keyword).cloned() {
            inner.shaders_to_release.remove(&keyword);
            cached.borrow_mut().count += 1;
            return cached.borrow().shader_program.clone();
        }

        let context = inner.context;
        let vs_text = vs.create_combined_vertex_shader(&context);
        let fs_text = fs.create_combined_fragment_shader(&context);

        let program = Rc::new(RefCell::new(ShaderProgram::new(
            vs,
            vs_text,
            fs,
            fs_text,
            attrs,
        )));

        let entry = Rc::new(RefCell::new(CachedShaderEntry {
            keyword: keyword.clone(),
            shader_program: program.clone(),
            count: 1,
            derived_keywords: Vec::new(),
            cache: Some(Rc::downgrade(&self.inner)),
        }));

        program.borrow_mut().cached_entry = Some(Rc::downgrade(&entry));
        inner.shaders.insert(keyword, entry);
        inner.number_of_shaders += 1;

        program
    }

    pub fn replace_shader_program(
        &self,
        old_program: Option<&Rc<RefCell<ShaderProgram>>>,
        options: ShaderProgramOptions,
    ) -> Rc<RefCell<ShaderProgram>> {
        if let Some(old) = old_program {
            old.borrow_mut().destroy();
        }
        self.get_shader_program(options)
    }

    pub fn release_shader_program(&self, program: &Rc<RefCell<ShaderProgram>>) {
        program.borrow_mut().destroy();
    }

    pub fn destroy_released_shader_programs(&self) {
        let mut inner = self.inner.borrow_mut();
        let to_release: Vec<_> = inner.shaders_to_release.values().cloned().collect();
        inner.shaders_to_release.clear();

        for entry in to_release {
            inner.destroy_shader_recursive(entry);
            if inner.number_of_shaders > 0 {
                inner.number_of_shaders -= 1;
            }
        }
    }

    pub fn create_derived_shader_program(
        &self,
        shader_program: &Rc<RefCell<ShaderProgram>>,
        keyword: &str,
        options: ShaderProgramOptions,
    ) -> Rc<RefCell<ShaderProgram>> {
        let parent_entry = shader_program
            .borrow()
            .cached_entry
            .as_ref()
            .and_then(|w| w.upgrade())
            .expect("Shader must be cached to create derived shader.");

        let derived_keyword = format!("{}{}", keyword, parent_entry.borrow().keyword);

        let vs = options.vertex_shader_source.to_shader_source();
        let fs = options.fragment_shader_source.to_shader_source();
        let attrs = options.attribute_locations;

        let mut inner = self.inner.borrow_mut();
        let context = inner.context;
        let vs_text = vs.create_combined_vertex_shader(&context);
        let fs_text = fs.create_combined_fragment_shader(&context);

        let derived_program = Rc::new(RefCell::new(ShaderProgram::new(
            vs,
            vs_text,
            fs,
            fs_text,
            attrs,
        )));

        let derived_entry = Rc::new(RefCell::new(CachedShaderEntry {
            keyword: derived_keyword.clone(),
            shader_program: derived_program.clone(),
            count: 0,
            derived_keywords: Vec::new(),
            cache: Some(Rc::downgrade(&self.inner)),
        }));

        derived_program.borrow_mut().cached_entry = Some(Rc::downgrade(&derived_entry));
        parent_entry.borrow_mut().derived_keywords.push(keyword.to_string());
        inner.shaders.insert(derived_keyword, derived_entry);

        derived_program
    }

    pub fn get_derived_shader_program(
        &self,
        shader_program: &Rc<RefCell<ShaderProgram>>,
        keyword: &str,
    ) -> Option<Rc<RefCell<ShaderProgram>>> {
        let parent_entry = shader_program
            .borrow()
            .cached_entry
            .as_ref()
            .and_then(|w| w.upgrade())?;

        let derived_keyword = format!("{}{}", keyword, parent_entry.borrow().keyword);
        let inner = self.inner.borrow();
        inner
            .shaders
            .get(&derived_keyword)
            .map(|e| e.borrow().shader_program.clone())
    }

    pub fn replace_derived_shader_program(
        &self,
        shader_program: &Rc<RefCell<ShaderProgram>>,
        keyword: &str,
        options: ShaderProgramOptions,
    ) -> Rc<RefCell<ShaderProgram>> {
        let parent_entry = shader_program
            .borrow()
            .cached_entry
            .as_ref()
            .and_then(|w| w.upgrade())
            .expect("Shader must be cached.");

        let derived_keyword = format!("{}{}", keyword, parent_entry.borrow().keyword);

        {
            let mut inner = self.inner.borrow_mut();
            if let Some(cached_derived) = inner.shaders.remove(&derived_keyword) {
                inner.destroy_shader_recursive(cached_derived);
                parent_entry
                    .borrow_mut()
                    .derived_keywords
                    .retain(|k| k != keyword);
            }
        }

        self.create_derived_shader_program(shader_program, keyword, options)
    }

    pub fn destroy(&self) {
        let mut inner = self.inner.borrow_mut();
        for entry in inner.shaders.values() {
            entry.borrow_mut().shader_program.borrow_mut().final_destroy();
        }
        inner.shaders.clear();
        inner.shaders_to_release.clear();
        inner.number_of_shaders = 0;
        inner.is_destroyed = true;
    }
}


fn parse_vertex_attributes(
    vertex_shader_text: &str,
    attribute_locations: &BTreeMap<String, u32>,
) -> BTreeMap<String, VertexAttributeDesc> {
    let mut attributes = BTreeMap::new();
    let mut auto_location = 0u32;

    for line in vertex_shader_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("in ") && trimmed.ends_with(';') {
            let parts: Vec<&str> = trimmed[3..trimmed.len() - 1].split_whitespace().collect();
            if parts.len() >= 2 {
                let glsl_type = parts[0].to_string();
                let name = parts[1].to_string();

                let index = if let Some(&loc) = attribute_locations.get(&name) {
                    loc
                } else {
                    while attribute_locations.values().any(|&v| v == auto_location)
                        || attributes.values().any(|desc: &VertexAttributeDesc| desc.index == auto_location)
                    {
                        auto_location += 1;
                    }
                    let res = auto_location;
                    auto_location += 1;
                    res
                };

                attributes.insert(
                    name.clone(),
                    VertexAttributeDesc {
                        name,
                        index,
                        glsl_type,
                    },
                );
            }
        }
    }

    attributes
}
