use std::cmp::Ordering;

/// Finds an item in a sorted slice using a comparator function.
///
/// Returns `Ok(index)` if found, or `Err(insert_index)` if not found.
pub fn binary_search<T, F: Fn(&T, &T) -> Ordering>(
    array: &[T],
    item_to_find: &T,
    comparator: F,
) -> Result<usize, usize> {
    if array.is_empty() {
        return Err(0);
    }

    let mut low = 0usize;
    let mut high = array.len() as isize - 1;

    while low as isize <= high {
        let i = (low as isize + high) / 2;
        let cmp = comparator(&array[i as usize], item_to_find);
        match cmp {
            Ordering::Less => {
                low = (i + 1) as usize;
            }
            Ordering::Greater => {
                high = i - 1;
            }
            Ordering::Equal => {
                return Ok(i as usize);
            }
        }
    }

    Err((high + 1) as usize)
}

/// Finds an item in a sorted slice, returning the index or bitwise complement if not found,
/// exactly matching CesiumJS `binarySearch` return value convention.
pub fn binary_search_cesium<T, F: Fn(&T, &T) -> Ordering>(
    array: &[T],
    item_to_find: &T,
    comparator: F,
) -> isize {
    match binary_search(array, item_to_find, comparator) {
        Ok(idx) => idx as isize,
        Err(insert_idx) => !(insert_idx as isize),
    }
}
