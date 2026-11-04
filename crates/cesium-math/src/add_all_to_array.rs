/// Adds all elements from source into the target vector.
/// If source is `None`, target remains unchanged.
pub fn add_all_to_array<T: Clone>(target: &mut Vec<T>, source: Option<&[T]>) {
    if let Some(src) = source {
        target.extend_from_slice(src);
    }
}
