/// Returns true if the optional value is Some.
#[inline]
pub fn defined<T>(value: Option<&T>) -> bool {
    value.is_some()
}
