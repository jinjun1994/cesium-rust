//! Shader building blocks: ShaderDestination, ShaderStruct, ShaderFunction matching CesiumJS `Renderer/Shader*.js` (Apache-2.0).

use cesium_math::DeveloperError;

/// Bit flag describing whether a variable or code block belongs in vertex, fragment, or both shaders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShaderDestination(pub u32);

impl ShaderDestination {
    pub const NONE: Self = Self(0);
    pub const VERTEX: Self = Self(1);
    pub const FRAGMENT: Self = Self(2);
    pub const BOTH: Self = Self(3);

    /// Checks if destination includes vertex shader.
    pub fn includes_vertex_shader(destination: Option<Self>) -> Result<bool, DeveloperError> {
        let dest = destination.ok_or_else(|| DeveloperError::new("destination is required."))?;
        Ok((dest.0 & Self::VERTEX.0) != 0)
    }

    /// Checks if destination includes fragment shader.
    pub fn includes_fragment_shader(destination: Option<Self>) -> Result<bool, DeveloperError> {
        let dest = destination.ok_or_else(|| DeveloperError::new("destination is required."))?;
        Ok((dest.0 & Self::FRAGMENT.0) != 0)
    }

    /// Computes the union of multiple destinations.
    pub fn union(destinations: &[Self]) -> Result<Self, DeveloperError> {
        if destinations.is_empty() {
            return Err(DeveloperError::new(
                "ShaderDestination.union requires at least one destination.",
            ));
        }
        let mut result = 0;
        for d in destinations {
            result |= d.0;
        }
        Ok(Self(result))
    }

    /// Computes the intersection of multiple destinations.
    pub fn intersection(destinations: &[Self]) -> Result<Self, DeveloperError> {
        if destinations.is_empty() {
            return Err(DeveloperError::new(
                "ShaderDestination.intersection requires at least one destination.",
            ));
        }
        let mut result = destinations[0].0;
        for d in &destinations[1..] {
            result &= d.0;
        }
        Ok(Self(result))
    }
}

/// A utility for dynamically generating a GLSL struct.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderStruct {
    pub name: String,
    pub fields: Vec<String>,
}

impl ShaderStruct {
    /// Creates a new ShaderStruct.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    /// Adds a field to the struct.
    pub fn add_field(&mut self, type_name: &str, identifier: &str) {
        self.fields.push(format!("    {} {};", type_name, identifier));
    }

    /// Generates lines of GLSL code.
    pub fn generate_glsl_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("struct {}", self.name));
        lines.push("{".to_string());
        if self.fields.is_empty() {
            lines.push("    float _empty;".to_string());
        } else {
            lines.extend(self.fields.clone());
        }
        lines.push("};".to_string());
        lines
    }
}

/// A utility for dynamically generating a GLSL function.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderFunction {
    pub signature: String,
    pub body: Vec<String>,
}

impl ShaderFunction {
    /// Creates a new ShaderFunction.
    pub fn new(signature: impl Into<String>) -> Self {
        Self {
            signature: signature.into(),
            body: Vec::new(),
        }
    }

    /// Adds a line to the function body.
    pub fn add_line(&mut self, line: impl AsRef<str>) {
        self.body.push(format!("    {}", line.as_ref()));
    }

    /// Adds multiple lines to the function body.
    pub fn add_lines(&mut self, lines: &[impl AsRef<str>]) {
        for line in lines {
            self.add_line(line);
        }
    }

    /// Generates lines of GLSL code.
    pub fn generate_glsl_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(self.signature.clone());
        lines.push("{".to_string());
        lines.extend(self.body.clone());
        lines.push("}".to_string());
        lines
    }
}


use std::collections::{BTreeMap, HashMap};
use crate::shader_source::{ShaderContextInfo, ShaderSource, ShaderSourceOptions};

fn get_attribute_location_count(glsl_type: &str) -> u32 {
    match glsl_type {
        "mat2" => 2,
        "mat3" => 3,
        "mat4" => 4,
        _ => 1,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ShaderParts {
    define_lines: Vec<String>,
    uniform_lines: Vec<String>,
    shader_lines: Vec<String>,
    varying_lines: Vec<String>,
    struct_ids: Vec<String>,
    function_ids: Vec<String>,
}

/// An object that makes it easier to build the text of a shader program.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderBuilder {
    position_attribute_line: Option<String>,
    next_attribute_location: u32,
    attribute_locations: BTreeMap<String, u32>,
    attribute_lines: Vec<String>,

    structs: HashMap<String, ShaderStruct>,
    functions: HashMap<String, ShaderFunction>,

    vertex_shader_parts: ShaderParts,
    fragment_shader_parts: ShaderParts,
}

impl Default for ShaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuiltShaderProgram {
    pub vertex_shader_text: String,
    pub fragment_shader_text: String,
    pub attribute_locations: BTreeMap<String, u32>,
}

impl ShaderBuilder {
    pub fn new() -> Self {
        Self {
            position_attribute_line: None,
            next_attribute_location: 1,
            attribute_locations: BTreeMap::new(),
            attribute_lines: Vec::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            vertex_shader_parts: ShaderParts::default(),
            fragment_shader_parts: ShaderParts::default(),
        }
    }

    pub fn attribute_locations(&self) -> &BTreeMap<String, u32> {
        &self.attribute_locations
    }

    pub fn vertex_uniform_lines(&self) -> &[String] {
        &self.vertex_shader_parts.uniform_lines
    }

    pub fn fragment_uniform_lines(&self) -> &[String] {
        &self.fragment_shader_parts.uniform_lines
    }

    pub fn vertex_define_lines(&self) -> &[String] {
        &self.vertex_shader_parts.define_lines
    }

    pub fn fragment_define_lines(&self) -> &[String] {
        &self.fragment_shader_parts.define_lines
    }

    pub fn vertex_lines(&self) -> &[String] {
        &self.vertex_shader_parts.shader_lines
    }

    pub fn fragment_lines(&self) -> &[String] {
        &self.fragment_shader_parts.shader_lines
    }

    pub fn add_define(
        &mut self,
        identifier: &str,
        value: Option<&str>,
        destination: Option<ShaderDestination>,
    ) {
        let dest = destination.unwrap_or(ShaderDestination::BOTH);
        let line = match value {
            Some(v) => format!("{identifier} {v}"),
            None => identifier.to_string(),
        };

        if (dest.0 & ShaderDestination::VERTEX.0) != 0 {
            self.vertex_shader_parts.define_lines.push(line.clone());
        }
        if (dest.0 & ShaderDestination::FRAGMENT.0) != 0 {
            self.fragment_shader_parts.define_lines.push(line);
        }
    }

    pub fn add_struct(
        &mut self,
        struct_id: &str,
        struct_name: &str,
        destination: ShaderDestination,
    ) {
        self.structs.insert(
            struct_id.to_string(),
            ShaderStruct::new(struct_name),
        );
        if (destination.0 & ShaderDestination::VERTEX.0) != 0 {
            self.vertex_shader_parts.struct_ids.push(struct_id.to_string());
        }
        if (destination.0 & ShaderDestination::FRAGMENT.0) != 0 {
            self.fragment_shader_parts.struct_ids.push(struct_id.to_string());
        }
    }

    pub fn add_struct_field(&mut self, struct_id: &str, field_type: &str, identifier: &str) {
        if let Some(s) = self.structs.get_mut(struct_id) {
            s.add_field(field_type, identifier);
        }
    }

    pub fn add_function(
        &mut self,
        function_name: &str,
        signature: &str,
        destination: ShaderDestination,
    ) {
        self.functions.insert(
            function_name.to_string(),
            ShaderFunction::new(signature),
        );
        if (destination.0 & ShaderDestination::VERTEX.0) != 0 {
            self.vertex_shader_parts.function_ids.push(function_name.to_string());
        }
        if (destination.0 & ShaderDestination::FRAGMENT.0) != 0 {
            self.fragment_shader_parts.function_ids.push(function_name.to_string());
        }
    }

    pub fn add_function_lines(&mut self, function_name: &str, lines: &[&str]) {
        if let Some(f) = self.functions.get_mut(function_name) {
            f.add_lines(lines);
        }
    }

    pub fn add_uniform(
        &mut self,
        field_type: &str,
        identifier: &str,
        destination: Option<ShaderDestination>,
    ) {
        let dest = destination.unwrap_or(ShaderDestination::BOTH);
        let line = format!("uniform {field_type} {identifier};");

        if (dest.0 & ShaderDestination::VERTEX.0) != 0 {
            self.vertex_shader_parts.uniform_lines.push(line.clone());
        }
        if (dest.0 & ShaderDestination::FRAGMENT.0) != 0 {
            self.fragment_shader_parts.uniform_lines.push(line);
        }
    }

    pub fn set_position_attribute(
        &mut self,
        field_type: &str,
        identifier: &str,
    ) -> Result<u32, DeveloperError> {
        if self.position_attribute_line.is_some() {
            return Err(DeveloperError::new(
                "setPositionAttribute() must be called exactly once for the attribute used for gl_Position.",
            ));
        }

        self.position_attribute_line = Some(format!("in {field_type} {identifier};"));
        self.attribute_locations.insert(identifier.to_string(), 0);
        Ok(0)
    }

    pub fn add_attribute(&mut self, field_type: &str, identifier: &str) -> u32 {
        let line = format!("in {field_type} {identifier};");
        self.attribute_lines.push(line);

        let location = self.next_attribute_location;
        self.attribute_locations.insert(identifier.to_string(), location);
        self.next_attribute_location += get_attribute_location_count(field_type);
        location
    }

    pub fn add_varying(&mut self, field_type: &str, identifier: &str, qualifier: Option<&str>) {
        let qual = match qualifier {
            Some(q) if !q.is_empty() => format!("{q} "),
            _ => String::new(),
        };

        let line = format!("{field_type} {identifier};");
        self.vertex_shader_parts
            .varying_lines
            .push(format!("{qual}out {line}"));
        self.fragment_shader_parts
            .varying_lines
            .push(format!("{qual}in {line}"));
    }

    pub fn add_vertex_lines(&mut self, lines: &[&str]) {
        for line in lines {
            self.vertex_shader_parts.shader_lines.push((*line).to_string());
        }
    }

    pub fn add_fragment_lines(&mut self, lines: &[&str]) {
        for line in lines {
            self.fragment_shader_parts.shader_lines.push((*line).to_string());
        }
    }

    fn generate_struct_lines(&self) -> (Vec<String>, Vec<String>) {
        let mut vertex_lines = Vec::new();
        let mut fragment_lines = Vec::new();

        for struct_id in &self.vertex_shader_parts.struct_ids {
            if let Some(s) = self.structs.get(struct_id) {
                vertex_lines.extend(s.generate_glsl_lines());
            }
        }

        for struct_id in &self.fragment_shader_parts.struct_ids {
            if let Some(s) = self.structs.get(struct_id) {
                fragment_lines.extend(s.generate_glsl_lines());
            }
        }

        (vertex_lines, fragment_lines)
    }

    fn generate_function_lines(&self) -> (Vec<String>, Vec<String>) {
        let mut vertex_lines = Vec::new();
        let mut fragment_lines = Vec::new();

        for func_id in &self.vertex_shader_parts.function_ids {
            if let Some(f) = self.functions.get(func_id) {
                vertex_lines.extend(f.generate_glsl_lines());
            }
        }

        for func_id in &self.fragment_shader_parts.function_ids {
            if let Some(f) = self.functions.get(func_id) {
                fragment_lines.extend(f.generate_glsl_lines());
            }
        }

        (vertex_lines, fragment_lines)
    }

    pub fn create_vertex_shader_source(&self) -> ShaderSource {
        let (struct_vertex_lines, _) = self.generate_struct_lines();
        let (func_vertex_lines, _) = self.generate_function_lines();

        let mut lines = Vec::new();
        if let Some(ref pos_attr) = self.position_attribute_line {
            lines.push(pos_attr.clone());
        }
        lines.extend(self.attribute_lines.clone());
        lines.extend(self.vertex_shader_parts.uniform_lines.clone());
        lines.extend(self.vertex_shader_parts.varying_lines.clone());
        lines.extend(struct_vertex_lines);
        lines.extend(func_vertex_lines);
        lines.extend(self.vertex_shader_parts.shader_lines.clone());

        let joined = lines.join("\n");
        let sources = if joined.is_empty() {
            Vec::new()
        } else {
            vec![joined]
        };

        ShaderSource::new(ShaderSourceOptions {
            defines: self.vertex_shader_parts.define_lines.clone(),
            sources,
            pick_color_qualifier: None,
            include_built_ins: Some(true),
        })
        .unwrap()
    }

    pub fn create_fragment_shader_source(&self) -> ShaderSource {
        let (_, struct_frag_lines) = self.generate_struct_lines();
        let (_, func_frag_lines) = self.generate_function_lines();

        let mut lines = Vec::new();
        lines.extend(self.fragment_shader_parts.uniform_lines.clone());
        lines.extend(self.fragment_shader_parts.varying_lines.clone());
        lines.extend(struct_frag_lines);
        lines.extend(func_frag_lines);
        lines.extend(self.fragment_shader_parts.shader_lines.clone());

        let joined = lines.join("\n");
        let sources = if joined.is_empty() {
            Vec::new()
        } else {
            vec![joined]
        };

        ShaderSource::new(ShaderSourceOptions {
            defines: self.fragment_shader_parts.define_lines.clone(),
            sources,
            pick_color_qualifier: None,
            include_built_ins: Some(true),
        })
        .unwrap()
    }

    pub fn build_shader_program(&self, context: &ShaderContextInfo) -> BuiltShaderProgram {
        let vs = self.create_vertex_shader_source();
        let fs = self.create_fragment_shader_source();

        BuiltShaderProgram {
            vertex_shader_text: vs.create_combined_vertex_shader(context),
            fragment_shader_text: fs.create_combined_fragment_shader(context),
            attribute_locations: self.attribute_locations.clone(),
        }
    }
}
