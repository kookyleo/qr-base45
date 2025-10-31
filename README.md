# qr-base45

[![Crates.io](https://img.shields.io/crates/v/qr-base45.svg)](https://crates.io/crates/qr-base45)
[![Docs.rs](https://docs.rs/qr-base45/badge.svg)](https://docs.rs/qr-base45)

Base45 encode/decode for arbitrary bytes per RFC 9285 (QR alphanumeric alphabet).

- 2 bytes -> 3 chars; 1 byte -> 2 chars
- Alphabet: `0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:`
- Public API: byte-oriented, no string reinterpretation

## Usage

```bash
cargo add qr-base45
```

```rust
use qr_base45::{encode, decode};

let data: &[u8] = &[0x01, 0x02, 0xFF];
let s = encode(data);
let back = decode(&s).unwrap();
assert_eq!(back, data);
```

## Notes
- This crate intentionally encodes/decodes arbitrary bytes, not UTF-8 text. If you have a text string, pass its bytes explicitly.
- Errors include invalid characters, dangling final character, and value overflow per RFC rules.

## License
Apache-2.0
