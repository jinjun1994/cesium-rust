use crate::DeveloperError;

#[derive(Clone, Default)]
struct VertexInfo {
    num_live_triangles: usize,
    time_stamp: usize,
    vertex_triangles: Vec<usize>,
}

/// Post-vertex-shader cache optimization algorithm (SIGGRAPH 2007).
pub struct Tipsify;

impl Tipsify {
    /// Calculates the average cache miss ratio (ACMR) for a given set of indices.
    pub fn calculate_acmr(
        indices: &[usize],
        maximum_index: Option<usize>,
        cache_size: Option<usize>,
    ) -> Result<f64, DeveloperError> {
        let num_indices = indices.len();
        if num_indices < 3 || !num_indices.is_multiple_of(3) {
            return Err(DeveloperError::new(
                "indices length must be a multiple of three.",
            ));
        }
        if let Some(max_idx) = maximum_index {
            if max_idx == 0 {
                return Err(DeveloperError::new(
                    "maximumIndex must be greater than zero.",
                ));
            }
        }
        let cache_sz = cache_size.unwrap_or(24);
        if cache_sz < 3 {
            return Err(DeveloperError::new("cacheSize must be greater than two."));
        }

        let max_idx = match maximum_index {
            Some(m) => m,
            None => *indices.iter().max().unwrap_or(&0),
        };

        let mut vertex_time_stamps = vec![0usize; max_idx + 1];
        let mut s = cache_sz + 1;

        for &idx in indices {
            if s.saturating_sub(vertex_time_stamps[idx]) > cache_sz {
                vertex_time_stamps[idx] = s;
                s += 1;
            }
        }

        let num_triangles = num_indices / 3;
        Ok((s - cache_sz + 1) as f64 / num_triangles as f64)
    }

    /// Optimizes triangle order for post-vertex-shader cache.
    pub fn tipsify(
        indices: &[usize],
        maximum_index: Option<usize>,
        cache_size: Option<usize>,
    ) -> Result<Vec<usize>, DeveloperError> {
        let num_indices = indices.len();
        if num_indices < 3 || !num_indices.is_multiple_of(3) {
            return Err(DeveloperError::new(
                "indices length must be a multiple of three.",
            ));
        }
        if let Some(max_idx) = maximum_index {
            if max_idx == 0 {
                return Err(DeveloperError::new(
                    "maximumIndex must be greater than zero.",
                ));
            }
        }
        let cache_sz = cache_size.unwrap_or(24);
        if cache_sz < 3 {
            return Err(DeveloperError::new("cacheSize must be greater than two."));
        }

        let max_idx_plus_one = match maximum_index {
            Some(m) => m + 1,
            None => indices.iter().max().map_or(1, |&m| m + 1),
        };

        let mut vertices = vec![VertexInfo::default(); max_idx_plus_one];
        let num_triangles = num_indices / 3;

        for triangle in 0..num_triangles {
            let i0 = indices[triangle * 3];
            let i1 = indices[triangle * 3 + 1];
            let i2 = indices[triangle * 3 + 2];

            vertices[i0].vertex_triangles.push(triangle);
            vertices[i0].num_live_triangles += 1;
            vertices[i1].vertex_triangles.push(triangle);
            vertices[i1].num_live_triangles += 1;
            vertices[i2].vertex_triangles.push(triangle);
            vertices[i2].num_live_triangles += 1;
        }

        let mut output_indices = Vec::with_capacity(num_indices);
        let mut triangle_emitted = vec![false; num_triangles];
        let mut dead_end = Vec::new();
        let mut cursor = 1usize;
        let mut s = cache_sz + 1;
        let mut f: isize = 0;

        let skip_dead_end = |vertices: &mut [VertexInfo],
                             dead_end: &mut Vec<usize>,
                             cursor: &mut usize,
                             max_idx_plus_one: usize|
         -> isize {
            while let Some(d) = dead_end.pop() {
                if vertices[d].num_live_triangles > 0 {
                    return d as isize;
                }
            }

            while *cursor < max_idx_plus_one {
                if vertices[*cursor].num_live_triangles > 0 {
                    *cursor += 1;
                    return (*cursor - 1) as isize;
                }
                *cursor += 1;
            }

            -1
        };

        while f != -1 {
            let mut one_ring = Vec::new();
            let vertex_idx = f as usize;

            let triangles = vertices[vertex_idx].vertex_triangles.clone();
            for triangle in triangles {
                if !triangle_emitted[triangle] {
                    triangle_emitted[triangle] = true;
                    let base = triangle * 3;
                    for j in 0..3 {
                        let idx = indices[base + j];
                        one_ring.push(idx);
                        dead_end.push(idx);
                        output_indices.push(idx);

                        let vertex = &mut vertices[idx];
                        vertex.num_live_triangles -= 1;
                        if s.saturating_sub(vertex.time_stamp) > cache_sz {
                            vertex.time_stamp = s;
                            s += 1;
                        }
                    }
                }
            }

            let mut n: isize = -1;
            let mut m: isize = -1;
            for &idx in &one_ring {
                let v = &vertices[idx];
                if v.num_live_triangles > 0 {
                    let mut p = 0isize;
                    if s.saturating_sub(v.time_stamp) + 2 * v.num_live_triangles <= cache_sz {
                        p = (s.saturating_sub(v.time_stamp)) as isize;
                    }
                    if p > m || m == -1 {
                        m = p;
                        n = idx as isize;
                    }
                }
            }

            if n == -1 {
                f = skip_dead_end(&mut vertices, &mut dead_end, &mut cursor, max_idx_plus_one);
            } else {
                f = n;
            }
        }

        Ok(output_indices)
    }
}
