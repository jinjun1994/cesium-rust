/// Appends a forward slash to the given URL or path if it doesn't already end with one.
pub fn append_forward_slash(url: &str) -> String {
    if url.ends_with('/') {
        url.to_string()
    } else {
        let mut s = String::with_capacity(url.len() + 1);
        s.push_str(url);
        s.push('/');
        s
    }
}
