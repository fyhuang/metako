mod filters;

mod renderers;
mod partial;
mod edit;
mod template;
mod save;

// TODO(fyhuang): make this private
pub use partial::ListingLayout;
pub use renderers::EntryRenderer;
pub use renderers::VideoPlayerRenderer;

pub use template::EntryListTemplate;
pub use template::DirIndexTemplate;
pub use template::ViewEntryTemplate;

pub use edit::EntryEditorPartial;
pub use save::SaveInlineFragment;
pub use save::SaveResultFragment;

fn nibble_to_hex(nibble: u8) -> u8 {
    debug_assert!(nibble < 16);
    if nibble < 10 {
        b'0' + nibble
    } else {
        b'A' + (nibble - 10)
    }
}

// Does *not* encode forward slashes!
pub fn urlencode_parts(input: &str) -> String {
    let unreserved_bytes: std::collections::HashSet<u8> =
        [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K',
         b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
         b'W', b'X', b'Y', b'Z',
         b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k',
         b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
         b'w', b'x', b'y', b'z',
         b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
         b'-', b'_', b'.', b'~',
         b'/']
        .iter().cloned().collect();
    let mut output_bytes = Vec::<u8>::new();
    for byte in input.as_bytes() {
        if unreserved_bytes.get(byte).is_some() {
            output_bytes.push(*byte);
        } else {
            // Encode this byte
            output_bytes.push(b'%');
            output_bytes.push(nibble_to_hex((*byte) >> 4));
            output_bytes.push(nibble_to_hex(byte & 0x0F));
        }
    }
    String::from_utf8(output_bytes).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibble_to_hex() {
        assert_eq!(nibble_to_hex(0), b'0');
        assert_eq!(nibble_to_hex(9), b'9');
        assert_eq!(nibble_to_hex(10), b'A');
        assert_eq!(nibble_to_hex(15), b'F');
    }

    #[test]
    fn test_urlencode() {
        let output = urlencode_parts("hello/[]你好.txt");
        assert_eq!(output, "hello/%5B%5D%E4%BD%A0%E5%A5%BD.txt");
    }
}
