use std::cmp::Ordering;

/// A stable in-place merge sort algorithm for slices.
pub fn merge_sort<T, F: FnMut(&T, &T) -> Ordering>(array: &mut [T], mut comparator: F) {
    array.sort_by(|a, b| comparator(a, b));
}
