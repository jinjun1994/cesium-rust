//! createElevationBandMaterial matching CesiumJS `Scene/createElevationBandMaterial.js` (Apache-2.0).

use cesium_math::{Color, DeveloperError};

pub const ELEVATION_BAND_MINIMUM_HEIGHT: f64 = -5906376425472.0;
pub const ELEVATION_BAND_MAXIMUM_HEIGHT: f64 = 5906376425472.0;
pub const ELEVATION_BAND_EMPTY_COLOR: Color = Color {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

/// An elevation band entry mapping a height to a color.
#[derive(Debug, Clone)]
pub struct ElevationBandEntry {
    pub height: Option<f64>,
    pub color: Option<Color>,
}

impl ElevationBandEntry {
    pub fn new(height: f64, color: Color) -> Self {
        Self {
            height: Some(height),
            color: Some(color),
        }
    }
}

/// A layer consisting of multiple elevation band entries with optional vertical extension.
#[derive(Debug, Clone, Default)]
pub struct ElevationBandLayer {
    pub entries: Vec<ElevationBandEntry>,
    pub extend_downwards: Option<bool>,
    pub extend_upwards: Option<bool>,
}

/// Generated texture data representing an elevation band material.
#[derive(Debug, Clone)]
pub struct ElevationBandMaterialResult {
    pub color_data: Vec<u8>,
    pub height_data: Vec<f64>,
}

#[derive(Debug, Clone)]
struct InternalEntry {
    height: f64,
    color: Color,
}

fn lerp_entry_color(height: f64, entry_before: &InternalEntry, entry_after: &InternalEntry) -> Color {
    let lerp_factor = if (entry_before.height - entry_after.height).abs() < 1e-9 {
        0.0
    } else {
        ((height - entry_before.height) / (entry_after.height - entry_before.height)).clamp(0.0, 1.0) as f32
    };
    Color::new(
        entry_before.color.red + (entry_after.color.red - entry_before.color.red) * lerp_factor,
        entry_before.color.green + (entry_after.color.green - entry_before.color.green) * lerp_factor,
        entry_before.color.blue + (entry_after.color.blue - entry_before.color.blue) * lerp_factor,
        entry_before.color.alpha + (entry_after.color.alpha - entry_before.color.alpha) * lerp_factor,
    )
}

fn remove_duplicates(mut entries: Vec<InternalEntry>) -> Vec<InternalEntry> {
    if entries.len() <= 2 {
        return entries;
    }

    // 1. Remove entries that have the same height as before and after
    let mut filtered = Vec::new();
    let n = entries.len();
    for i in 0..n {
        let same_height_as_prev = if i > 0 {
            entries[i].height == entries[i - 1].height
        } else {
            true
        };
        let same_height_as_next = if i < n - 1 {
            entries[i].height == entries[i + 1].height
        } else {
            true
        };
        if !same_height_as_prev || !same_height_as_next {
            filtered.push(entries[i].clone());
        }
    }
    entries = filtered;

    // 2. Remove entries that have the same color as before and after
    let mut filtered_color = Vec::new();
    let n = entries.len();
    for i in 0..n {
        let same_color_as_prev = if i > 0 {
            entries[i].color == entries[i - 1].color
        } else {
            false
        };
        let same_color_as_next = if i < n - 1 {
            entries[i].color == entries[i + 1].color
        } else {
            false
        };
        if !same_color_as_prev || !same_color_as_next {
            filtered_color.push(entries[i].clone());
        }
    }
    entries = filtered_color;

    // 3. Remove entries that have the same height AND color as the entry before
    let mut final_entries: Vec<InternalEntry> = Vec::new();
    for entry in entries {
        if let Some(prev) = final_entries.last() {
            if prev.height == entry.height && prev.color == entry.color {
                continue;
            }
        }
        final_entries.push(entry);
    }

    final_entries
}

fn preprocess(layers: &[ElevationBandLayer]) -> Result<Vec<Vec<InternalEntry>>, DeveloperError> {
    if layers.is_empty() {
        return Err(DeveloperError::new("layers must have length > 0."));
    }

    let mut layered_entries = Vec::new();

    for layer in layers {
        if layer.entries.is_empty() {
            return Err(DeveloperError::new("entries must have length > 0."));
        }

        let mut entries = Vec::new();
        for entry in &layer.entries {
            let height = entry
                .height
                .ok_or_else(|| DeveloperError::new("entry requires a height."))?;
            let color = entry
                .color
                .ok_or_else(|| DeveloperError::new("entry requires a color."))?;

            let clamped_h = height.clamp(ELEVATION_BAND_MINIMUM_HEIGHT, ELEVATION_BAND_MAXIMUM_HEIGHT);

            // premultiplied alpha
            let mut pre_color = color;
            pre_color.red *= pre_color.alpha;
            pre_color.green *= pre_color.alpha;
            pre_color.blue *= pre_color.alpha;

            entries.push(InternalEntry {
                height: clamped_h,
                color: pre_color,
            });
        }

        // Sort ascending by height
        entries.sort_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal));

        let mut extend_downwards = layer.extend_downwards.unwrap_or(false);
        let mut extend_upwards = layer.extend_upwards.unwrap_or(false);

        if entries.len() == 1 && !extend_downwards && !extend_upwards {
            extend_downwards = true;
            extend_upwards = true;
        }

        if extend_downwards {
            let first_col = entries[0].color;
            entries.insert(0, InternalEntry {
                height: ELEVATION_BAND_MINIMUM_HEIGHT,
                color: first_col,
            });
        }
        if extend_upwards {
            let last_col = entries.last().unwrap().color;
            entries.push(InternalEntry {
                height: ELEVATION_BAND_MAXIMUM_HEIGHT,
                color: last_col,
            });
        }

        let entries = remove_duplicates(entries);
        layered_entries.push(entries);
    }

    Ok(layered_entries)
}

fn blend_colors(a: Color, b: Color) -> Color {
    let mut res = Color {
        red: b.red * (1.0 - a.alpha) + a.red,
        green: b.green * (1.0 - a.alpha) + a.green,
        blue: b.blue * (1.0 - a.alpha) + a.blue,
        alpha: b.alpha * (1.0 - a.alpha) + a.alpha,
    };
    res.red = res.red.clamp(0.0, 1.0);
    res.green = res.green.clamp(0.0, 1.0);
    res.blue = res.blue.clamp(0.0, 1.0);
    res.alpha = res.alpha.clamp(0.0, 1.0);
    res
}

const EMPTY_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.0);

fn create_layered_entries(
    layers: &[ElevationBandLayer],
) -> Result<Vec<InternalEntry>, DeveloperError> {
    let layered_entries = preprocess(layers)?;

    let mut entries_accum_curr: Vec<InternalEntry>;
    let mut entries_accum_next: Vec<InternalEntry> = Vec::new();

    for entries in layered_entries {
        entries_accum_curr = entries_accum_next;
        entries_accum_next = Vec::new();

        let mut idx = 0;
        let mut accum_idx = 0;
        let entries_len = entries.len();
        let accum_len = entries_accum_curr.len();

        while idx < entries_len || accum_idx < accum_len {
            let entry = if idx < entries_len { Some(&entries[idx]) } else { None };
            let prev_entry = if idx > 0 && idx <= entries_len { Some(&entries[idx - 1]) } else { None };
            let next_entry = if idx + 1 < entries_len { Some(&entries[idx + 1]) } else { None };

            let entry_accum = if accum_idx < accum_len { Some(&entries_accum_curr[accum_idx]) } else { None };
            let prev_entry_accum = if accum_idx > 0 && accum_idx <= accum_len { Some(&entries_accum_curr[accum_idx - 1]) } else { None };
            let next_entry_accum = if accum_idx + 1 < accum_len { Some(&entries_accum_curr[accum_idx + 1]) } else { None };

            if let (Some(e), Some(ea)) = (entry, entry_accum) {
                if (e.height - ea.height).abs() < 1e-9 {
                    let is_split_accum = next_entry_accum.is_some_and(|nea| (ea.height - nea.height).abs() < 1e-9);
                    let is_start_accum = prev_entry_accum.is_none();
                    let is_end_accum = next_entry_accum.is_none();

                    let is_split = next_entry.is_some_and(|ne| (e.height - ne.height).abs() < 1e-9);
                    let is_start = prev_entry.is_none();
                    let is_end = next_entry.is_none();

                    if is_split_accum {
                        let nea = next_entry_accum.unwrap();
                        if is_split {
                            let ne = next_entry.unwrap();
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(ne.color, nea.color) });
                        } else if is_start {
                            entries_accum_next.push(InternalEntry { height: e.height, color: ea.color });
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, nea.color) });
                        } else if is_end {
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                            entries_accum_next.push(InternalEntry { height: e.height, color: nea.color });
                        } else {
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, nea.color) });
                        }
                    } else if is_start_accum {
                        if is_split {
                            let ne = next_entry.unwrap();
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(ne.color, ea.color) });
                        } else if is_end {
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                            entries_accum_next.push(InternalEntry { height: e.height, color: ea.color });
                        } else if is_start {
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                        } else {
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                        }
                    } else if is_end_accum {
                        if is_split {
                            let ne = next_entry.unwrap();
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                            entries_accum_next.push(InternalEntry { height: e.height, color: ne.color });
                        } else if is_start {
                            entries_accum_next.push(InternalEntry { height: e.height, color: ea.color });
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                        } else if is_end {
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                        } else {
                            entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                        }
                    } else if is_split {
                        let ne = next_entry.unwrap();
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(ne.color, ea.color) });
                    } else if is_start {
                        entries_accum_next.push(InternalEntry { height: e.height, color: ea.color });
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                    } else if is_end {
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                        entries_accum_next.push(InternalEntry { height: e.height, color: ea.color });
                    } else {
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, ea.color) });
                    }

                    idx += if is_split { 2 } else { 1 };
                    accum_idx += if is_split_accum { 2 } else { 1 };
                    continue;
                } else if let (Some(pea), true) = (prev_entry_accum, e.height < ea.height) {
                    let color_below = lerp_entry_color(e.height, pea, ea);
                    if prev_entry.is_none() {
                        entries_accum_next.push(InternalEntry { height: e.height, color: color_below });
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, color_below) });
                    } else if next_entry.is_none() {
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, color_below) });
                        entries_accum_next.push(InternalEntry { height: e.height, color: color_below });
                    } else {
                        entries_accum_next.push(InternalEntry { height: e.height, color: blend_colors(e.color, color_below) });
                    }
                    idx += 1;
                    continue;
                } else if let (Some(pe), true) = (prev_entry, ea.height < e.height) {
                    let color_above = lerp_entry_color(ea.height, pe, e);
                    if prev_entry_accum.is_none() {
                        entries_accum_next.push(InternalEntry { height: ea.height, color: color_above });
                        entries_accum_next.push(InternalEntry { height: ea.height, color: blend_colors(color_above, ea.color) });
                    } else if next_entry_accum.is_none() {
                        entries_accum_next.push(InternalEntry { height: ea.height, color: blend_colors(color_above, ea.color) });
                        entries_accum_next.push(InternalEntry { height: ea.height, color: color_above });
                    } else {
                        entries_accum_next.push(InternalEntry { height: ea.height, color: blend_colors(color_above, ea.color) });
                    }
                    accum_idx += 1;
                    continue;
                }
            }

            if let Some(e) = entry {
                if entry_accum.is_none() || e.height < entry_accum.unwrap().height {
                    if let Some(ea) = entry_accum {
                        if prev_entry_accum.is_none() && next_entry.is_none() {
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                            entries_accum_next.push(InternalEntry { height: e.height, color: EMPTY_COLOR });
                            entries_accum_next.push(InternalEntry { height: ea.height, color: EMPTY_COLOR });
                        } else {
                            entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                        }
                    } else if let (Some(pea), true) = (prev_entry_accum, prev_entry.is_none()) {
                        entries_accum_next.push(InternalEntry { height: pea.height, color: EMPTY_COLOR });
                        entries_accum_next.push(InternalEntry { height: e.height, color: EMPTY_COLOR });
                        entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                    } else {
                        entries_accum_next.push(InternalEntry { height: e.height, color: e.color });
                    }
                    idx += 1;
                    continue;
                }
            }

            if let Some(ea) = entry_accum {
                if entry.is_none() || ea.height < entry.unwrap().height {
                    entries_accum_next.push(InternalEntry { height: ea.height, color: ea.color });
                    accum_idx += 1;
                    continue;
                }
            }
        }
    }

    Ok(remove_duplicates(entries_accum_next))
}

/// Creates an elevation band material mapping layered elevation bands to height and color textures.
pub fn create_elevation_band_material(
    layers: &[ElevationBandLayer],
) -> Result<ElevationBandMaterialResult, DeveloperError> {
    let all_entries = create_layered_entries(layers)?;
    let count = all_entries.len();

    let mut color_data = Vec::with_capacity(count * 4);
    let mut height_data = Vec::with_capacity(count);

    for entry in all_entries {
        let r = (entry.color.red * 255.0).round().clamp(0.0, 255.0) as u8;
        let g = (entry.color.green * 255.0).round().clamp(0.0, 255.0) as u8;
        let b = (entry.color.blue * 255.0).round().clamp(0.0, 255.0) as u8;
        let a = (entry.color.alpha * 255.0).round().clamp(0.0, 255.0) as u8;

        color_data.extend_from_slice(&[r, g, b, a]);
        height_data.push(entry.height);
    }

    Ok(ElevationBandMaterialResult {
        color_data,
        height_data,
    })
}
