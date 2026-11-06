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
