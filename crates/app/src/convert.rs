use fast_codec_codec::Transform;

/// The other box's text, and the status line; an empty status means success.
#[derive(Debug, PartialEq, Eq)]
pub struct Converted {
    pub text: String,
    pub status: String,
}

impl Converted {
    fn ok(text: String) -> Self {
        Self {
            text,
            status: String::new(),
        }
    }

    fn failed(status: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            status: status.into(),
        }
    }
}

pub fn encode(transform: &dyn Transform, decoded: &str) -> Converted {
    if decoded.is_empty() {
        return Converted::ok(String::new());
    }
    // Typed text is always valid UTF-8; U+FFFD is how invalid input shows up.
    if decoded.contains(char::REPLACEMENT_CHARACTER) {
        return Converted::failed("contains U+FFFD (\u{FFFD}), likely from a lossy conversion");
    }
    Converted::ok(transform.encode(decoded.as_bytes()))
}

pub fn decode(transform: &dyn Transform, encoded: &str) -> Converted {
    if encoded.is_empty() {
        return Converted::ok(String::new());
    }
    match transform.decode(encoded) {
        None => Converted::failed(format!("{} can't be decoded", transform.name())),
        Some(Err(e)) => Converted::failed(e.to_string()),
        Some(Ok(bytes)) => match String::from_utf8(bytes) {
            Ok(text) => Converted::ok(text),
            Err(_) => Converted::failed("decoded bytes aren't valid UTF-8"),
        },
    }
}

#[cfg(test)]
mod tests {
    use fast_codec_codec::{Base64, DecodeError};

    use super::*;

    #[test]
    fn round_trip() {
        let decoded = "héllo, wörld ✓";
        let encoded = encode(&Base64, decoded);
        assert_eq!(encoded.status, "");
        assert_eq!(
            decode(&Base64, &encoded.text),
            Converted::ok(decoded.into())
        );
    }

    #[test]
    fn encodes_known_value() {
        assert_eq!(encode(&Base64, "hello"), Converted::ok("aGVsbG8=".into()));
    }

    #[test]
    fn decodes_known_value() {
        assert_eq!(decode(&Base64, "aGVsbG8="), Converted::ok("hello".into()));
    }

    #[test]
    fn empty_input_gives_empty_output_and_status() {
        assert_eq!(encode(&Base64, ""), Converted::ok(String::new()));
        assert_eq!(decode(&Base64, ""), Converted::ok(String::new()));
    }

    #[test]
    fn non_utf8_decode_clears_decoded_with_warning() {
        // "/w==" is the single byte 0xFF.
        let out = decode(&Base64, "/w==");
        assert_eq!(out.text, "");
        assert_eq!(out.status, "decoded bytes aren't valid UTF-8");
    }

    #[test]
    fn replacement_char_clears_encoded_with_warning() {
        let out = encode(&Base64, "abc\u{FFFD}def");
        assert_eq!(out.text, "");
        assert!(out.status.contains("U+FFFD"), "{}", out.status);
    }

    #[test]
    fn decode_error_clears_decoded_and_shows_error_text() {
        let out = decode(&Base64, "aGV*");
        assert_eq!(out.text, "");
        assert_eq!(out.status, DecodeError::InvalidChar('*').to_string());
    }

    #[test]
    fn one_way_transform_clears_decoded_with_error() {
        struct OneWay;
        impl Transform for OneWay {
            fn name(&self) -> &'static str {
                "OneWay"
            }
            fn encode(&self, _: &[u8]) -> String {
                "x".into()
            }
        }
        let out = decode(&OneWay, "x");
        assert_eq!(out.text, "");
        assert_eq!(out.status, "OneWay can't be decoded");
    }
}
