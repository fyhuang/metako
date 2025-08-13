use base64::{Engine as _, engine::general_purpose};
use mtk::filter_sort::FilterSortOptions;

/// Encode FilterSortOptions as base64 for URL parameters
pub fn to_base64(options: &FilterSortOptions) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(options)?;
    Ok(general_purpose::STANDARD.encode(json.as_bytes()))
}

/// Decode FilterSortOptions from base64 URL parameter
pub fn from_base64(encoded: &str) -> Result<FilterSortOptions, Box<dyn std::error::Error>> {
    let decoded = general_purpose::STANDARD.decode(encoded)?;
    let json = String::from_utf8(decoded)?;
    let options = serde_json::from_str(&json)?;
    Ok(options)
}

/// Generate a sort URL for the given sort field
pub fn sort_url(options: &FilterSortOptions, sort_by_str: &str) -> String {
    let new_options = options.with_sort(sort_by_str);
    let encoded = to_base64(&new_options).unwrap_or_default();
    format!("?sort={}", encoded)
}

pub fn parse_sort_options(sort_param: Option<&str>) -> FilterSortOptions {
    match sort_param {
        Some(encoded) => {
            from_base64(encoded).unwrap_or_else(|e| {
                println!("Failed to parse sort options: {}, using defaults", e);
                FilterSortOptions::default()
            })
        }
        None => FilterSortOptions::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtk::filter_sort::{SortBy, SortOrder};

    #[test]
    fn test_base64_serialization() {
        let options = FilterSortOptions {
            sort_by: SortBy::Rating,
            sort_order: SortOrder::Desc,
            filter_file_types: Some(vec!["video".to_string()]),
            filter_rating: Some((3, 5)),
        };

        let encoded = to_base64(&options).unwrap();
        let decoded = from_base64(&encoded).unwrap();
        
        assert_eq!(options.sort_by, decoded.sort_by);
        assert_eq!(options.sort_order, decoded.sort_order);
        assert_eq!(options.filter_file_types, decoded.filter_file_types);
        assert_eq!(options.filter_rating, decoded.filter_rating);
    }

    #[test]
    fn test_invalid_base64_returns_error() {
        let result = from_base64("invalid-base64");
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_url_generation() {
        let options = FilterSortOptions {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Asc,
            ..Default::default()
        };
        
        let url = sort_url(&options, "name");
        assert!(url.starts_with("?sort="));
        
        // Should be able to decode back
        let encoded = url.strip_prefix("?sort=").unwrap();
        let decoded = from_base64(encoded).unwrap();
        assert_eq!(decoded.sort_by, SortBy::Name);
        assert_eq!(decoded.sort_order, SortOrder::Desc); // Should toggle
    }

    #[test]
    fn test_parse_sort_options() {
        let options = FilterSortOptions {
            sort_by: SortBy::Size,
            sort_order: SortOrder::Desc,
            ..Default::default()
        };
        
        let encoded = to_base64(&options).unwrap();
        let parsed = parse_sort_options(Some(&encoded));
        
        assert_eq!(parsed.sort_by, SortBy::Size);
        assert_eq!(parsed.sort_order, SortOrder::Desc);
    }

    #[test]
    fn test_parse_sort_options_none() {
        let parsed = parse_sort_options(None);
        assert_eq!(parsed, FilterSortOptions::default());
    }
}