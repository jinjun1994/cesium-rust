//! Label and LabelCollection matching CesiumJS `Scene/LabelCollection.js` and `Scene/Label.js` (Apache-2.0).

use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    Cartesian2, Cartesian3, Color, DistanceDisplayCondition, HeightReference,
    HorizontalOrigin, LabelStyle, NearFarScalar, VerticalOrigin,
};

static ENABLE_RTL_DETECTION: AtomicBool = AtomicBool::new(false);

/// A single label representation managed by LabelCollection.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub id: Option<String>,
    pub show: bool,
    pub position: Cartesian3,
    pub text: String,
    pub rendered_text: String,
    pub font: String,
    pub style: LabelStyle,
    pub fill_color: Color,
    pub outline_color: Color,
    pub outline_width: f64,
    pub show_background: bool,
    pub background_color: Color,
    pub background_padding: Cartesian2,
    pub scale: f64,
    pub horizontal_origin: HorizontalOrigin,
    pub vertical_origin: VerticalOrigin,
    pub eye_offset: Cartesian3,
    pub pixel_offset: Cartesian2,
    pub height_reference: HeightReference,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub pixel_offset_scale_by_distance: Option<NearFarScalar>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub clamped_position: Option<Cartesian3>,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            id: None,
            show: true,
            position: Cartesian3::ZERO,
            text: String::new(),
            rendered_text: String::new(),
            font: "30px sans-serif".to_string(),
            style: LabelStyle::Fill,
            fill_color: Color::WHITE,
            outline_color: Color::BLACK,
            outline_width: 1.0,
            show_background: false,
            background_color: Color::new(0.165, 0.165, 0.165, 0.8),
            background_padding: Cartesian2::new(7.0, 5.0),
            scale: 1.0,
            horizontal_origin: HorizontalOrigin::Left,
            vertical_origin: VerticalOrigin::Baseline,
            eye_offset: Cartesian3::ZERO,
            pixel_offset: Cartesian2::ZERO,
            height_reference: HeightReference::None,
            translucency_by_distance: None,
            pixel_offset_scale_by_distance: None,
            scale_by_distance: None,
            distance_display_condition: None,
            disable_depth_test_distance: None,
            clamped_position: None,
        }
    }
}

impl Label {
    /// Sets global right-to-left text detection.
    pub fn set_enable_right_to_left_detection(enable: bool) {
        ENABLE_RTL_DETECTION.store(enable, Ordering::SeqCst);
    }

    /// Gets global right-to-left text detection.
    pub fn get_enable_right_to_left_detection() -> bool {
        ENABLE_RTL_DETECTION.load(Ordering::SeqCst)
    }

    /// Updates text and recomputes `rendered_text`.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        let filtered = Self::filter_unsupported_characters(text);
        if Self::get_enable_right_to_left_detection() {
            self.rendered_text = Self::reverse_rtl(&filtered);
        } else {
            self.rendered_text = filtered;
        }
    }

    /// Updates position clamping according to height reference.
    pub fn update_clamping(&mut self) {
        if self.height_reference != HeightReference::None {
            self.clamped_position = Some(self.position);
        } else {
            self.clamped_position = None;
        }
    }

    /// Filters unsupported unicode control characters.
    pub fn filter_unsupported_characters(text: &str) -> String {
        text.chars()
            .filter(|&c| {
                !matches!(
                    c,
                    '\u{0000}'..='\u{0008}'
                        | '\u{000E}'..='\u{001F}'
                        | '\u{00AD}'
                        | '\u{202A}'..='\u{206F}'
                        | '\u{200B}'..='\u{200F}'
                )
            })
            .collect()
    }

    /// Checks equality with another optional label.
    pub fn equals(&self, other: Option<&Label>) -> bool {
        match other {
            Some(o) => self == o,
            None => false,
        }
    }

    /// Reverses text for RTL display matching CesiumJS algorithm.
    pub fn reverse_rtl(value: &str) -> String {
        let lines: Vec<&str> = value.split('\n').collect();
        let mut result = String::new();

        for (line_idx, text) in lines.iter().enumerate() {
            if !text.chars().any(is_rtl_char) {
                result.push_str(text);
                if line_idx < lines.len() - 1 {
                    result.push('\n');
                }
                continue;
            }

            let first_char_rtl = text.chars().next().map(is_rtl_char).unwrap_or(false);
            let parsed_text = convert_text_to_types(text);

            let mut line = String::new();
            let mut splice_pointer = 0usize;

            for word_index in 0..parsed_text.len() {
                let sub_text = &parsed_text[word_index];
                let reverse = if sub_text.sub_type == TextType::Brackets {
                    reverse_brackets(&sub_text.word)
                } else {
                    sub_text.word.chars().rev().collect()
                };

                if first_char_rtl {
                    if sub_text.sub_type == TextType::Rtl {
                        line = format!("{reverse}{line}");
                        splice_pointer = 0;
                    } else if sub_text.sub_type == TextType::Ltr {
                        line = splice_word(&line, splice_pointer, &sub_text.word);
                        splice_pointer += sub_text.word.chars().count();
                    } else if sub_text.sub_type == TextType::Weak || sub_text.sub_type == TextType::Brackets {
                        if word_index > 0 && sub_text.sub_type == TextType::Weak && parsed_text[word_index - 1].sub_type == TextType::Brackets {
                            line = format!("{reverse}{line}");
                        } else if word_index > 0 && parsed_text[word_index - 1].sub_type == TextType::Rtl {
                            line = format!("{reverse}{line}");
                            splice_pointer = 0;
                        } else if parsed_text.len() > word_index + 1 {
                            if parsed_text[word_index + 1].sub_type == TextType::Rtl {
                                line = format!("{reverse}{line}");
                                splice_pointer = 0;
                            } else {
                                line = splice_word(&line, splice_pointer, &sub_text.word);
                                splice_pointer += sub_text.word.chars().count();
                            }
                        } else {
                            line = splice_word(&line, 0, &reverse);
                        }
                    }
                } else if sub_text.sub_type == TextType::Rtl {
                    line = splice_word(&line, splice_pointer, &reverse);
                } else if sub_text.sub_type == TextType::Ltr {
                    line.push_str(&sub_text.word);
                    splice_pointer = line.chars().count();
                } else if sub_text.sub_type == TextType::Weak || sub_text.sub_type == TextType::Brackets {
                    if word_index > 0 {
                        if parsed_text[word_index - 1].sub_type == TextType::Rtl {
                            if parsed_text.len() > word_index + 1 {
                                if parsed_text[word_index + 1].sub_type == TextType::Rtl {
                                    line = splice_word(&line, splice_pointer, &reverse);
                                } else {
                                    line.push_str(&sub_text.word);
                                    splice_pointer = line.chars().count();
                                }
                            } else {
                                line.push_str(&sub_text.word);
                            }
                        } else {
                            line.push_str(&sub_text.word);
                            splice_pointer = line.chars().count();
                        }
                    } else {
                        line.push_str(&sub_text.word);
                        splice_pointer = line.chars().count();
                    }
                }
            }

            result.push_str(&line);
            if line_idx < lines.len() - 1 {
                result.push('\n');
            }
        }

        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TextType {
    Ltr,
    Rtl,
    Weak,
    Brackets,
}

#[derive(Debug, Clone)]
struct ParsedSubText {
    sub_type: TextType,
    word: String,
}

fn is_rtl_char(c: char) -> bool {
    matches!(
        c,
        '\u{05D0}'..='\u{05EA}'
            | '\u{0600}'..='\u{06FF}'
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
    )
}

fn is_ltr_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

fn is_bracket(c: char) -> bool {
    matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
}

fn reverse_brackets(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '(' => ')',
            ')' => '(',
            '[' => ']',
            ']' => '[',
            '{' => '}',
            '}' => '{',
            '<' => '>',
            '>' => '<',
            other => other,
        })
        .collect()
}

fn convert_text_to_types(text: &str) -> Vec<ParsedSubText> {
    let mut parsed = Vec::new();
    let mut word = String::new();
    let mut last_type = TextType::Ltr;

    for (idx, c) in text.chars().enumerate() {
        let current_type = if is_rtl_char(c) {
            TextType::Rtl
        } else if is_ltr_char(c) {
            TextType::Ltr
        } else if is_bracket(c) {
            TextType::Brackets
        } else {
            TextType::Weak
        };

        if idx == 0 {
            last_type = current_type;
        }

        if last_type == current_type && current_type != TextType::Brackets {
            word.push(c);
        } else {
            if !word.is_empty() {
                parsed.push(ParsedSubText {
                    sub_type: last_type,
                    word,
                });
            }
            last_type = current_type;
            word = c.to_string();
        }
    }

    if !word.is_empty() {
        parsed.push(ParsedSubText {
            sub_type: last_type,
            word,
        });
    }

    parsed
}

fn splice_word(result: &str, pointer: usize, word: &str) -> String {
    let chars: Vec<char> = result.chars().collect();
    let p = pointer.min(chars.len());
    let left: String = chars[..p].iter().collect();
    let right: String = chars[p..].iter().collect();
    format!("{left}{word}{right}")
}

/// A renderable collection of labels.
#[derive(Clone, Debug, Default)]
pub struct LabelCollection {
    pub labels: Vec<Label>,
    destroyed: bool,
}

impl LabelCollection {
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            destroyed: false,
        }
    }

    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Label> {
        self.labels.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Label> {
        self.labels.get_mut(index)
    }

    pub fn add(&mut self, label: Label) -> usize {
        self.labels.push(label);
        self.labels.len() - 1
    }

    pub fn add_default(&mut self) -> &mut Label {
        self.labels.push(Label::default());
        let last_idx = self.labels.len() - 1;
        &mut self.labels[last_idx]
    }

    pub fn contains(&self, label: &Label) -> bool {
        self.labels.iter().any(|l| l.equals(Some(label)))
    }

    pub fn remove_label(&mut self, label: &Label) -> bool {
        if let Some(pos) = self.labels.iter().position(|l| l.equals(Some(label))) {
            self.labels.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.labels.len() {
            self.labels.remove(index);
        }
    }

    pub fn remove_all(&mut self) {
        self.labels.clear();
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn destroy(&mut self) {
        self.remove_all();
        self.destroyed = true;
    }
}
