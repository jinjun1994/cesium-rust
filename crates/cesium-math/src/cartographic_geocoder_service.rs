use crate::Cartesian3;

/// Result from geocoding a location query.
#[derive(Debug, Clone, PartialEq)]
pub struct GeocoderResult {
    pub display_name: String,
    pub destination: Cartesian3,
}

/// Geocodes queries containing longitude and latitude coordinates and an optional height.
/// Query format: `longitude latitude (height)` with longitude/latitude in degrees and height in meters,
/// or NSEW coordinate strings such as `35N 75W`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CartographicGeocoderService;

impl CartographicGeocoderService {
    pub fn new() -> Self {
        Self
    }

    /// Geocodes a query string into a list of results.
    pub fn geocode(&self, query: &str) -> Vec<GeocoderResult> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let tokens: Vec<&str> = trimmed
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.len() != 2 && tokens.len() != 3 {
            return Vec::new();
        }

        let num0 = tokens[0].parse::<f64>();
        let num1 = tokens[1].parse::<f64>();

        // Check if both are pure numbers
        if let (Ok(lon), Ok(lat)) = (num0, num1) {
            let height = if tokens.len() == 3 {
                match tokens[2].parse::<f64>() {
                    Ok(h) => h,
                    Err(_) => return Vec::new(),
                }
            } else {
                300.0
            };

            if let Ok(destination) = Cartesian3::from_degrees(lon, lat, Some(height), None) {
                return vec![GeocoderResult {
                    display_name: query.to_string(),
                    destination,
                }];
            }
        }

        // Check if both are NSEW coordinates
        let mut lon_val: Option<f64> = None;
        let mut lat_val: Option<f64> = None;
        let mut nsew_count = 0;

        for token in &tokens[..2] {
            if let Some((val, dir)) = parse_nsew_token(token) {
                nsew_count += 1;
                match dir {
                    'n' => lat_val = Some(val),
                    's' => lat_val = Some(-val),
                    'e' => lon_val = Some(val),
                    'w' => lon_val = Some(-val),
                    _ => {}
                }
            }
        }

        if nsew_count == 2 {
            if let (Some(lon), Some(lat)) = (lon_val, lat_val) {
                let height = if tokens.len() == 3 {
                    match tokens[2].parse::<f64>() {
                        Ok(h) => h,
                        Err(_) => return Vec::new(),
                    }
                } else {
                    300.0
                };

                if let Ok(destination) = Cartesian3::from_degrees(lon, lat, Some(height), None) {
                    return vec![GeocoderResult {
                        display_name: query.to_string(),
                        destination,
                    }];
                }
            }
        }

        Vec::new()
    }
}

fn parse_nsew_token(token: &str) -> Option<(f64, char)> {
    let s = token.trim();
    if s.len() < 2 {
        return None;
    }
    let last_char = s.chars().last()?;
    let dir = last_char.to_ascii_lowercase();
    if dir != 'n' && dir != 's' && dir != 'e' && dir != 'w' {
        return None;
    }
    let num_str = &s[..s.len() - last_char.len_utf8()];
    let val = num_str.parse::<f64>().ok()?;
    Some((val, dir))
}
