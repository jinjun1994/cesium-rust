//! Lightweight pure-Rust XML DOM parser for GPX and KML data sources.

use crate::DeveloperError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XmlNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<XmlNode>,
    pub text: String,
}

impl XmlNode {
    pub fn local_name(&self) -> &str {
        self.tag.rsplit(':').next().unwrap_or(&self.tag)
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    pub fn child(&self, name: &str) -> Option<&XmlNode> {
        self.children
            .iter()
            .find(|c| c.tag == name || c.local_name() == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a XmlNode> + 'a {
        self.children
            .iter()
            .filter(move |c| c.tag == name || c.local_name() == name)
    }

    pub fn child_text(&self, name: &str) -> Option<String> {
        self.child(name).map(|c| c.all_text())
    }

    pub fn all_text(&self) -> String {
        if self.children.is_empty() {
            return self.text.trim().to_string();
        }
        let mut out = self.text.clone();
        for c in &self.children {
            out.push_str(&c.all_text());
        }
        out.trim().to_string()
    }
}

pub(crate) fn parse_xml(input: &str) -> Result<XmlNode, DeveloperError> {
    let input = input.trim_start_matches('\u{feff}');
    let mut parser = XmlParser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    parser.parse_document()
}

struct XmlParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn starts_with(&self, s: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(s)
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_until(&mut self, needle: &[u8]) -> bool {
        if let Some(idx) = self.bytes[self.pos..]
            .windows(needle.len())
            .position(|w| w == needle)
        {
            self.pos += idx + needle.len();
            true
        } else {
            self.pos = self.bytes.len();
            false
        }
    }

    fn parse_document(&mut self) -> Result<XmlNode, DeveloperError> {
        loop {
            self.skip_ws();
            if self.is_eof() {
                return Err(DeveloperError::new("Empty XML document."));
            }
            if self.starts_with(b"<?") {
                if !self.skip_until(b"?>") {
                    return Err(DeveloperError::new("Unterminated XML processing instruction."));
                }
            } else if self.starts_with(b"<!--") {
                if !self.skip_until(b"-->") {
                    return Err(DeveloperError::new("Unterminated XML comment."));
                }
            } else if self.starts_with(b"<!DOCTYPE") || self.starts_with(b"<!doctype") {
                if !self.skip_until(b">") {
                    return Err(DeveloperError::new("Unterminated XML DOCTYPE."));
                }
            } else if self.starts_with(b"<") {
                return self.parse_element();
            } else {
                return Err(DeveloperError::new("Invalid XML: expected '<' at root."));
            }
        }
    }

    fn parse_name(&mut self) -> Result<String, DeveloperError> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':' || b == b'.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(DeveloperError::new("Invalid XML: expected name."));
        }
        Ok(String::from_utf8_lossy(&self.bytes[start..self.pos]).into_owned())
    }

    fn parse_attr_value(&mut self) -> Result<String, DeveloperError> {
        let quote = self
            .peek()
            .filter(|&b| b == b'"' || b == b'\'')
            .ok_or_else(|| DeveloperError::new("Invalid XML: expected attribute quote."))?;
        self.pos += 1;
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == quote {
                let raw = String::from_utf8_lossy(&self.bytes[start..self.pos]);
                self.pos += 1;
                return Ok(unescape_xml(&raw));
            }
            self.pos += 1;
        }
        Err(DeveloperError::new("Invalid XML: unterminated attribute value."))
    }

    fn parse_element(&mut self) -> Result<XmlNode, DeveloperError> {
        if !self.starts_with(b"<") {
            return Err(DeveloperError::new("Invalid XML: expected '<'."));
        }
        self.pos += 1;
        let tag = self.parse_name()?;
        let mut attributes = HashMap::new();

        loop {
            self.skip_ws();
            if self.starts_with(b"/>") {
                self.pos += 2;
                return Ok(XmlNode {
                    tag,
                    attributes,
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            if self.starts_with(b">") {
                self.pos += 1;
                break;
            }
            if self.is_eof() {
                return Err(DeveloperError::new("Invalid XML: unterminated start tag."));
            }
            let attr_name = self.parse_name()?;
            self.skip_ws();
            if self.peek() == Some(b'=') {
                self.pos += 1;
                self.skip_ws();
                let attr_val = self.parse_attr_value()?;
                attributes.insert(attr_name, attr_val);
            } else {
                attributes.insert(attr_name, String::new());
            }
        }

        let mut children = Vec::new();
        let mut text = String::new();

        loop {
            if self.is_eof() {
                return Err(DeveloperError::new(format!(
                    "Invalid XML: unclosed tag <{}>.",
                    tag
                )));
            }
            if self.starts_with(b"</") {
                self.pos += 2;
                let close_tag = self.parse_name()?;
                self.skip_ws();
                if self.peek() != Some(b'>') {
                    return Err(DeveloperError::new("Invalid XML: expected '>' in closing tag."));
                }
                self.pos += 1;
                if close_tag != tag {
                    return Err(DeveloperError::new(format!(
                        "Invalid XML: mismatched closing tag </{}> for <{}>.",
                        close_tag, tag
                    )));
                }
                return Ok(XmlNode {
                    tag,
                    attributes,
                    children,
                    text,
                });
            } else if self.starts_with(b"<!--") {
                if !self.skip_until(b"-->") {
                    return Err(DeveloperError::new("Unterminated XML comment."));
                }
            } else if self.starts_with(b"<![CDATA[") {
                self.pos += 9;
                let start = self.pos;
                if let Some(idx) = self.bytes[self.pos..].windows(3).position(|w| w == b"]]>") {
                    text.push_str(&String::from_utf8_lossy(&self.bytes[start..self.pos + idx]));
                    self.pos += idx + 3;
                } else {
                    return Err(DeveloperError::new("Unterminated CDATA section."));
                }
            } else if self.starts_with(b"<") {
                children.push(self.parse_element()?);
            } else {
                let start = self.pos;
                while let Some(b) = self.peek() {
                    if b == b'<' {
                        break;
                    }
                    self.pos += 1;
                }
                let raw = String::from_utf8_lossy(&self.bytes[start..self.pos]);
                text.push_str(&unescape_xml(&raw));
            }
        }
    }
}

fn unescape_xml(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        if let Some(semi) = after.find(';') {
            let entity = &after[..semi];
            match entity {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                    if let Ok(code) = u32::from_str_radix(&entity[2..], 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                _ if entity.starts_with('#') => {
                    if let Ok(code) = entity[1..].parse::<u32>() {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                _ => {
                    out.push('&');
                    out.push_str(entity);
                    out.push(';');
                }
            }
            rest = &after[semi + 1..];
        } else {
            out.push('&');
            rest = after;
            break;
        }
    }
    out.push_str(rest);
    out
}
