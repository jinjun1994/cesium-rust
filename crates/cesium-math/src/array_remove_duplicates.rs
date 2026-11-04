use crate::EPSILON10;

/// Removes adjacent duplicate values in a slice of values.
///
/// * `values` - The slice of values.
/// * `equals_epsilon` - Function to compare values with an epsilon: `equals_epsilon(&left, &right, epsilon) -> bool`.
/// * `wrap_around` - Compare the last value in the slice against the first value. If equal, the last is removed.
/// * `removed_indices` - Optional `Vec<usize>` to store indices of removed items.
pub fn array_remove_duplicates<T: Clone, F: Fn(&T, &T, f64) -> bool>(
    values: &[T],
    equals_epsilon: F,
    wrap_around: bool,
    mut removed_indices: Option<&mut Vec<usize>>,
) -> Vec<T> {
    let length = values.len();
    if length < 2 {
        return values.to_vec();
    }

    let mut cleaned_values: Option<Vec<T>> = None;
    let mut last_clean_index = 0;
    let mut removed_index_lci: isize = -1;
    let mut v0 = &values[0];

    for i in 1..length {
        let v1 = &values[i];
        if equals_epsilon(v0, v1, EPSILON10) {
            if cleaned_values.is_none() {
                cleaned_values = Some(values[0..i].to_vec());
                last_clean_index = i - 1;
                removed_index_lci = 0;
            }
            if let Some(ref mut indices) = removed_indices {
                indices.push(i);
            }
        } else {
            if let Some(ref mut cleaned) = cleaned_values {
                cleaned.push(v1.clone());
                last_clean_index = i;
                if let Some(ref indices) = removed_indices {
                    removed_index_lci = indices.len() as isize;
                }
            }
            v0 = v1;
        }
    }

    if wrap_around && equals_epsilon(&values[0], &values[length - 1], EPSILON10) {
        if let Some(ref mut indices) = removed_indices {
            if cleaned_values.is_some() {
                let idx = if removed_index_lci < 0 {
                    0
                } else {
                    removed_index_lci as usize
                };
                indices.insert(idx, last_clean_index);
            } else {
                indices.push(length - 1);
            }
        }

        if let Some(ref mut cleaned) = cleaned_values {
            cleaned.pop();
        } else {
            cleaned_values = Some(values[0..length - 1].to_vec());
        }
    }

    cleaned_values.unwrap_or_else(|| values.to_vec())
}
