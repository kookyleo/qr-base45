//! qr-base45: Base45 encoder/decoder for arbitrary bytes (RFC 9285) using the QR alphanumeric alphabet.
//! - Encoding groups: 2 bytes -> 3 chars; 1 byte -> 2 chars.
//! - Alphabet: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"
//! - Public API encodes &[u8] -> String and decodes &str -> Vec<u8>.

#[derive(Debug, thiserror::Error)]
pub enum Base45Error {
    #[error("invalid base45 character")]
    InvalidChar,
    #[error("dangling character group")]
    Dangling,
    #[error("value overflow")]
    Overflow,
}

/// Base45 alphabet as per RFC 9285
pub const BASE45_ALPHABET: &[u8; 45] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

#[inline]
fn b45_val(ch: u8) -> Option<u16> {
    match ch {
        b'0'..=b'9' => Some((ch - b'0') as u16),
        b'A'..=b'Z' => Some(10 + (ch - b'A') as u16),
        b' ' => Some(36),
        b'$' => Some(37),
        b'%' => Some(38),
        b'*' => Some(39),
        b'+' => Some(40),
        b'-' => Some(41),
        b'.' => Some(42),
        b'/' => Some(43),
        b':' => Some(44),
        _ => None,
    }
}

/// Encode arbitrary bytes into a Base45 string.
/// Groups of 2 bytes produce 3 characters; a final single byte produces 2 characters.
pub fn encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() * 3).div_ceil(2));
    let mut i = 0;
    while i + 1 < input.len() {
        let x = (input[i] as u16) * 256 + (input[i + 1] as u16);
        let c = x % 45; // least significant digit
        let x = x / 45;
        let b = x % 45;
        let a = x / 45; // most significant digit (0..=8)
                        // Base45 outputs least-significant digit first
        out.push(BASE45_ALPHABET[c as usize] as char);
        out.push(BASE45_ALPHABET[b as usize] as char);
        out.push(BASE45_ALPHABET[a as usize] as char);
        i += 2;
    }
    if i < input.len() {
        let x = input[i] as u16;
        let b = x % 45;
        let a = x / 45;
        // Base45 outputs least-significant digit first for single byte too
        out.push(BASE45_ALPHABET[b as usize] as char);
        out.push(BASE45_ALPHABET[a as usize] as char);
    }
    out
}

/// Decode a Base45 string back to raw bytes.
/// Accepts only the RFC 9285 alphabet; returns errors for invalid chars, dangling final char, or overflow.
pub fn decode(s: &str) -> Result<Vec<u8>, Base45Error> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i + 2 < bytes.len() {
        // Input is least-significant digit first: c (lsd), b, a (msd)
        let c0 = b45_val(bytes[i]).ok_or(Base45Error::InvalidChar)? as u32;
        let c1 = b45_val(bytes[i + 1]).ok_or(Base45Error::InvalidChar)? as u32;
        let c2 = b45_val(bytes[i + 2]).ok_or(Base45Error::InvalidChar)? as u32;
        let x: u32 = c2 * 45 * 45 + c1 * 45 + c0; // 0..(45^3 - 1)
        if x > 65535 {
            return Err(Base45Error::Overflow);
        }
        out.push((x / 256) as u8);
        out.push((x % 256) as u8);
        i += 3;
    }
    if i < bytes.len() {
        if i + 1 >= bytes.len() {
            // Single trailing character: report InvalidChar if it's not in alphabet, otherwise Dangling
            if b45_val(bytes[i]).is_none() { return Err(Base45Error::InvalidChar); }
            return Err(Base45Error::Dangling);
        }
        let c0 = b45_val(bytes[i]).ok_or(Base45Error::InvalidChar)? as u32;
        let c1 = b45_val(bytes[i + 1]).ok_or(Base45Error::InvalidChar)? as u32;
        let x: u32 = c1 * 45 + c0; // 0..(45^2 - 1)
        if x > 255 {
            return Err(Base45Error::Overflow);
        }
        out.push(x as u8);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let cases: &[&[u8]] = &[
            b"",
            b"A",
            b"AB",
            b"Hello, world!",
            &[0x00],
            &[0x00, 0x01, 0xFF, 0x80, 0x7F],
        ];
        for &case in cases {
            let s = encode(case);
            let dec = decode(&s).unwrap();
            assert_eq!(case, dec.as_slice());
        }
    }

    #[test]
    fn known_vectors() {
        // Base45 uses least-significant digit first (lsd-first): output order is c, b, a.
        // For a 2-byte group [u, v], form x = u*256 + v, then:
        // c = x % 45; x /= 45; b = x % 45; a = x / 45; and output chars are [c, b, a].
        // For a 1-byte group [u], b = u % 45; a = u / 45; and output chars are [b, a].
        // Edge cases at boundaries
        // [0x00, 0x00] -> x = 0; digits: c=0, b=0, a=0; output lsd-first -> "000"
        assert_eq!(encode(&[0x00, 0x00]), "000");

        // From RFC examples and common vectors
        assert_eq!(encode(b"AB"), "BB8");
        assert_eq!(encode(b"Hello!!"), "%69 VD92EX0");
        assert_eq!(encode(b"base-45"), "UJCLQE7W581");
        assert_eq!(encode(b"ietf!"), "QED8WEX0");

        assert_eq!(decode("BB8").unwrap(), b"AB");
        assert_eq!(decode("QED8WEX0").unwrap(), b"ietf!");
    }

    #[test]
    fn errors() {
        // Error categories under test:
        // - InvalidChar: character not in RFC 9285 alphabet
        // - Dangling: incomplete group (e.g., single trailing valid character)
        // - Overflow: numeric value exceeds maximum for the group
        // Invalid characters and structural errors
        assert!(matches!(decode("\t"), Err(Base45Error::InvalidChar))); // '\t' not in Base45 alphabet
        assert!(matches!(decode("\n"), Err(Base45Error::InvalidChar))); // '\n' not in Base45 alphabet
        // Overflow cases
        // 3-char group with max digits -> value > 65535
        assert!(matches!(decode(":::"), Err(Base45Error::Overflow))); // ':::' -> 44*45^2 + 44*45 + 44 = 91124 > 65535
        // 2-char group producing >255
        assert!(matches!(decode("ZZ"), Err(Base45Error::Overflow))); // 'ZZ' -> 35*45 + 35 = 1610 > 255

        assert!(matches!(decode("A"), Err(Base45Error::Dangling))); // single valid char -> incomplete group
        assert!(matches!(decode("😀"), Err(Base45Error::InvalidChar))); // not in Base45 alphabet
    }
}
