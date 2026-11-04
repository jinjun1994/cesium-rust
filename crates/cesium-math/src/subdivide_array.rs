/// Subdivides an array into a number of smaller, equal sized arrays.
pub fn subdivide_array<T: Clone>(
    array: &[T],
    number_of_arrays: usize,
) -> Result<Vec<Vec<T>>, &'static str> {
    if number_of_arrays == 0 {
        return Err("numberOfArrays must be greater than 0.");
    }
    if array.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::with_capacity(number_of_arrays);
    let len = array.len();
    let mut i = 0;
    let mut remaining = number_of_arrays;

    while i < len && remaining > 0 {
        let size = (len - i).div_ceil(remaining);
        remaining -= 1;
        let end = (i + size).min(len);
        result.push(array[i..end].to_vec());
        i += size;
    }

    Ok(result)
}
