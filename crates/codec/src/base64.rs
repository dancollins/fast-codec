use base64::DecodeError as Base64Error;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD_PAD_INDIFFERENT, URL_SAFE_PAD_INDIFFERENT};

use crate::{DecodeError, Transform};

/// Encodes standard and padded. Decodes either alphabet, with or without padding,
/// ignoring ASCII whitespace.
pub struct Base64;

impl Transform for Base64 {
    fn name(&self) -> &'static str {
        "Base64"
    }

    fn encode(&self, input: &[u8]) -> String {
        STANDARD_PAD_INDIFFERENT.encode(input)
    }

    fn decode(&self, input: &str) -> Option<Result<Vec<u8>, DecodeError>> {
        Some(decode(input))
    }
}

fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    let mut compact = String::with_capacity(input.len());
    let (mut standard, mut url_safe) = (false, false);
    for c in input.chars() {
        match c {
            c if c.is_ascii_whitespace() => continue,
            'A'..='Z' | 'a'..='z' | '0'..='9' | '=' => {}
            '+' | '/' => standard = true,
            '-' | '_' => url_safe = true,
            _ => return Err(DecodeError::InvalidChar(c)),
        }
        compact.push(c);
    }
    if standard && url_safe {
        return Err(DecodeError::MixedAlphabets);
    }
    let engine = if url_safe {
        URL_SAFE_PAD_INDIFFERENT
    } else {
        STANDARD_PAD_INDIFFERENT
    };
    // Every byte left is ASCII, so `as char` is exact.
    engine.decode(compact).map_err(|e| match e {
        Base64Error::InvalidByte(_, b'=') | Base64Error::InvalidPadding => {
            DecodeError::InvalidPadding
        }
        Base64Error::InvalidByte(_, b) => DecodeError::InvalidChar(b as char),
        Base64Error::InvalidLastSymbol { symbol, .. } => {
            DecodeError::InvalidLastChar(symbol as char)
        }
        Base64Error::InvalidLength(_) => DecodeError::Incomplete,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4648 section 10.
    const VECTORS: [(&str, &str); 7] = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];

    fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
        Base64.decode(input).expect("base64 decodes")
    }

    #[test]
    fn encodes_rfc_vectors() {
        for (plain, encoded) in VECTORS {
            assert_eq!(Base64.encode(plain.as_bytes()), encoded, "{plain:?}");
        }
    }

    #[test]
    fn decodes_rfc_vectors() {
        for (plain, encoded) in VECTORS {
            assert_eq!(decode(encoded).unwrap(), plain.as_bytes(), "{encoded:?}");
        }
    }

    #[test]
    fn decodes_without_padding() {
        for (plain, encoded) in VECTORS {
            let unpadded = encoded.trim_end_matches('=');
            assert_eq!(decode(unpadded).unwrap(), plain.as_bytes(), "{unpadded:?}");
        }
    }

    #[test]
    fn ignores_whitespace() {
        assert_eq!(
            decode(" Zm9v\r\nYmFy\n\tZm9v\x0cYg==\n").unwrap(),
            b"foobarfoob"
        );
        assert_eq!(decode(" \n\t").unwrap(), b"");
    }

    #[test]
    fn decodes_url_safe() {
        assert_eq!(decode("-_-_").unwrap(), [0xfb, 0xff, 0xbf]);
        assert_eq!(decode("-_8").unwrap(), [0xfb, 0xff]);
        assert_eq!(decode("+/+/").unwrap(), [0xfb, 0xff, 0xbf]);
    }

    #[test]
    fn rejects_mixed_alphabets() {
        assert_eq!(decode("+/-_"), Err(DecodeError::MixedAlphabets));
        assert_eq!(decode("ab+c\nde_f"), Err(DecodeError::MixedAlphabets));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(decode("Zm9v!"), Err(DecodeError::InvalidChar('!')));
        assert_eq!(decode("Zm9vé"), Err(DecodeError::InvalidChar('é')));
    }

    #[test]
    fn rejects_malformed_input() {
        assert_eq!(decode("Zm=9v"), Err(DecodeError::InvalidPadding));
        assert_eq!(decode("Zm9v="), Err(DecodeError::InvalidPadding));
        assert_eq!(decode("Zm9vY"), Err(DecodeError::Incomplete));
        assert_eq!(decode("Zh"), Err(DecodeError::InvalidLastChar('h')));
    }

    #[test]
    fn round_trips_non_utf8() {
        let bytes = [0x00, 0x80, 0xc3, 0x28, 0xfe, 0xff];
        assert_eq!(decode(&Base64.encode(&bytes)).unwrap(), bytes);
    }
}
