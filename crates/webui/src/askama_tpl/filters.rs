pub fn urlencode_parts(input: &str, _: &dyn askama::Values) -> askama::Result<String> {
    let encoded = super::urlencode_parts(input);
    Ok(encoded)
}
