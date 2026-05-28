String Obfuscation
==================

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/obfstr.svg)](https://crates.io/crates/obfstr)
[![docs.rs](https://docs.rs/obfstr/badge.svg)](https://docs.rs/obfstr)
[![Build status](https://github.com/CasualX/obfstr/workflows/CI/badge.svg)](https://github.com/CasualX/obfstr/actions)

Compiletime string constant obfuscation for Rust.

The string constant itself is embedded in obfuscated form and deobfuscated locally.
This reference to a temporary value must be used in the same statement it was generated.
See the documentation for more advanced use cases.

Looking for obfuscating formatting strings? See `fmtools` ([github](https://github.com/CasualX/fmtools), [crates.io](https://crates.io/crates/fmtools), [docs.rs](https://docs.rs/fmtools/0.1.2/fmtools/)) with the optional dependency `obfstr` enabled to automatically apply string obfuscation to formatting strings.

Examples
--------

The `obfstr!` macro returns the deobfuscated string as a temporary value:

```rust
use obfany::obfstr as s;
assert_eq!(s!("Hello 🌍"), "Hello 🌍");
```

The `wide!` macro provides compiletime utf16 string constants:

```rust
let expected = &['W' as u16, 'i' as u16, 'd' as u16, 'e' as u16, 0];
assert_eq!(obfany::wide!("Wide\0"), expected);
```

The `random!` macro provides compiletime random values:

```rust
const RND: i32 = obfany::random!(u8) as i32;
assert!(RND >= 0 && RND <= 255);
```

Compiletime random values are based on `file!()`, `line!()`, `column!()` and a fixed seed to ensure reproducibility.
This fixed seed is stored as text in the environment variable `OBFANY_SEED` and can be changed as desired.

The `obfnum!` macro obfuscates numeric constants:

```rust
let secret = obfany::obfnum!(0x1234_u32);
assert_eq!(secret, 0x1234_u32);
```

Always use a typed literal suffix (e.g. `0x1234_u32`, `-9999_i64`, `3.14_f32`) to ensure correctness.
Supports all primitive numeric types: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `usize`, `isize`, `f32`, `f64`.
The value is stored XOR-encrypted in the binary and decrypted at runtime.

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
