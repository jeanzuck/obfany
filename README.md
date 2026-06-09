# obfany

> Forked from [obfstr](https://github.com/CasualX/obfstr) by CasualX — extended with `obfnum!`,
> modernised to Rust 2024 edition (1.85+), and expanded test coverage.

[![Rust](https://img.shields.io/badge/rust-stable%201.85+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/obfany.svg)](https://crates.io/crates/obfany)
[![docs.rs](https://docs.rs/obfany/badge.svg)](https://docs.rs/obfany)

Compiletime string and number constant obfuscation for Rust — `no_std` and zero dependencies.

Values are stored XOR-encrypted in the binary and decrypted locally at runtime. Static analysis tools
see only ciphertext — plaintext constants never appear in the data section.

---

## Quick Start

```rust
// Cargo.toml
// [dependencies]
// obfany = "0.1"

use obfany::obfstr as s;

// String literals are embedded in obfuscated form
assert_eq!(s!("Hello 🌍"), "Hello 🌍");

// Numbers are stored XOR-encrypted
let secret: u32 = obfany::obfnum!(0xDEAD_BEEF_u32);
assert_eq!(secret, 0xDEAD_BEEF_u32);

// Compiletime random values
const SALT: u64 = obfany::random!(u64);
```

---

## Usage

### String Obfuscation — `obfstr!`

Four syntax forms cover every use case. The deobfuscated temporary lives only for the enclosing statement.

**Inline expression** — use directly where a `&str` is expected:

```rust
use obfany::obfstr as s;
println!("{}", s!("Hello world"));
```

**Multi-let** — obfuscate several strings into local bindings:

```rust
use obfany::obfstr as s;
s! {
    let hello = "Hello";
    let world = "World";
}
assert_eq!(hello, "Hello");
assert_eq!(world, "World");
```

**Assign to outer variable** — deobfuscate into an uninitialised binding:

```rust
use obfany::obfstr as s;
let greeting;
s!(greeting = "Hello");
assert_eq!(greeting, "Hello");
```

**Write into a caller-provided buffer** — zero heap allocations:

```rust
use obfany::obfstr as s;
let mut buf = [0u8; 16];
let s = s!(buf <- "hello");
assert_eq!(s, "hello");
```

### CStr Obfuscation — `obfcstr!`

Same syntax as `obfstr!`, produces `&CStr`:

```rust
use std::ffi::CStr;
use obfany::obfcstr as cstr;

const GREETING: &CStr = c"Hello CStr";
assert_eq!(cstr!(GREETING).to_str().unwrap(), "Hello CStr");
```

### Owned String — `obfstring!`

Returns `String` when you need an owned value:

```rust
use obfany::obfstring;
let s: String = obfstring!("I am owned");
assert_eq!(s, "I am owned");
```

### Byte Slice Obfuscation — `obfbytes!`

Same syntax as `obfstr!`, produces `&[u8]`:

```rust
use obfany::obfbytes;
assert_eq!(obfbytes!(b"raw bytes"), b"raw bytes");
```

### Wide String Obfuscation — `obfwide!`

Obfuscates and encodes as UTF-16 (`&[u16]`):

```rust
use obfany::obfwide;
assert_eq!(obfwide!("Wide"), obfany::wide!("Wide"));
```

### Number Obfuscation — `obfnum!`

XOR-encrypts an integer or float constant. Always use a typed literal suffix.

```rust
use obfany::obfnum;

// Integers
assert_eq!(obfnum!(0x1234_u32),   0x1234_u32);
assert_eq!(obfnum!(-9999_i64),   -9999_i64);
assert_eq!(obfnum!(u128::MAX),    u128::MAX);

// Floats
assert_eq!(obfnum!(3.14159265358979_f64), std::f64::consts::PI);
assert_eq!(obfnum!(1.0_f32),             1.0_f32);
```

Supported types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`,
`i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `f32`, `f64`.

### Compiletime Random — `random!`

Produces deterministic compiletime values using `file!()`, `line!()`, `column!()`,
and a global seed. Same input → same output across builds.

```rust
use obfany::random;

const RND: u32 = random!(u32);
const FLAG: bool = random!(bool);

// Extra seeds disambiguate calls on the same line
let a = random!(u64, "lhs");
let b = random!(u64, "rhs");
assert_ne!(a, b);
```

Supported types: `u8`, `u16`, `u32`, `u64`, `usize`, `i8`, `i16`, `i32`, `i64`,
`isize`, `bool`, `f32`, `f64`.

Floats are in `[1.0, 2.0)`. Integers fill their full range.

### UTF-16 Encoding — `wide!`

Encodes a string literal as a compiletime `&[u16; N]`:

```rust
use obfany::wide;
assert_eq!(wide!("Rust\0"), &[b'R' as u16, b'u' as u16, b's' as u16, b't' as u16, 0]);
```

Handles Unicode, escape sequences, and raw string literals.

### Control Flow Obfuscation — `obfstmt!`

Randomises the execution order of a statement sequence. Each iteration dispatches
via a match-on-key so the compiled control flow looks unrelated to the source order.

```rust
use obfany::obfstmt;

let mut x = 0;
obfstmt! {
    x = 2;
    x *= 22;
    x -= 12;
    x /= 3;
}
assert_eq!(x, 10);
```

Key collisions (two statements hashing to the same dispatch key) are detected at
compile time — the build will fail with a clear message.

### Cross-Reference Obfuscation — `xref!` / `xref_mut!`

Hides the reference to a `static` in the disassembly. The pointer is reconstructed
through opaque arithmetic so the original symbol is not directly visible.

```rust
use obfany::xref;

static SECRET: i32 = 42;
let p: &i32 = xref!(&SECRET);
assert_eq!(*p, 42);
```

`xref!` works with `&str` and `&[u8]` as a lightweight alternative to `obfstr!`:

```rust
assert_eq!(xref!("Hello"), "Hello");
assert_eq!(xref!(b"bytes"), b"bytes");
```

For mutable statics, use `xref_mut!`:

```rust
static mut COUNTER: i32 = 0;
let p: &mut i32 = obfany::xref_mut!(unsafe { &mut *(&raw mut COUNTER) });
```

### Compiletime Hashing — `hash!` / `murmur3!`

`hash!` — DJB2-xor hash of a string literal:

```rust
use obfany::hash;
assert_eq!(hash!("Hello World"), 0x6E4A573D);
```

`murmur3!` — MurmurHash3 (32-bit) keyed hash of a byte slice:

```rust
use obfany::murmur3;
let h: u32 = murmur3!(b"data", 0x12345678);
```

### Compiletime Substring Search — `position!`

Finds a needle in a haystack at compile time. Panics at compile time if not found — ideal for
pooling multiple strings in a single obfuscated blob:

```rust
use obfany::{obfstr, position};

const POOL: &str = concat!("Foo", "Bar", "Baz");
obfstr! { let pool = POOL; }

let foo = &pool[position!(POOL, "Foo")];
let bar = &pool[position!(POOL, "Bar")];
let baz = &pool[position!(POOL, "Baz")];
```

---

## Reproducible Builds

The global RNG seed is controlled by the `OBFANY_SEED` environment variable at compile time.
When unset it defaults to `"FIXED"`. Changing the seed forces recompilation of all dependents.

```sh
OBFANY_SEED="my-custom-seed" cargo build --release
```

The seed value itself — not a hash — is embedded, so builds with the same seed produce
identical binaries.

---

## API Reference

### Macros

| Macro | Signature | Description |
|-------|-----------|-------------|
| `obfstr!` | `($s:expr)` → `&str` | Obfuscate a string literal |
| `obfstr!` | `{ let $name = $s; … }` | Multi-let form |
| `obfstr!` | `($name = $s)` | Assign to outer variable |
| `obfstr!` | `($buf <- $s)` | Write into buffer, return `&str` |
| `obfcstr!` | (same forms) → `&CStr` | Obfuscate a CStr literal |
| `obfstring!` | `($s:expr)` → `String` | Obfuscate into owned `String` |
| `obfbytes!` | (same forms) → `&[u8]` | Obfuscate a byte slice |
| `obfwide!` | (same forms) → `&[u16]` | Obfuscate a UTF-16 string |
| `obfnum!` | `($val:expr)` → `T` | Obfuscate a numeric literal |
| `random!` | `($ty $(, $seed)*)` → `T` | Compiletime random value |
| `wide!` | `($s:expr)` → `&[u16; N]` | UTF-16 encode at compile time |
| `obfstmt!` | `{ $($stmt;)* }` | Control flow obfuscation |
| `xref!` | `($e:expr)` → `&T` | Obfuscate a static reference |
| `xref_mut!` | `($e:expr)` → `&mut T` | Obfuscate a mutable static reference |
| `hash!` | `($s:expr)` → `u32` | DJB2-xor hash at compile time |
| `murmur3!` | `($s:expr $(, $seed)?)` → `u32` | MurmurHash3 at compile time |
| `position!` | `($haystack, $needle)` → `Range<usize>` | Substring search at compile time |

### Constants

| Constant | Type | Description |
|----------|------|-------------|
| `SEED` | `u64` | Active RNG seed (derived from `OBFANY_SEED`) |

---

## How It Works

```
┌─────────────────────────────────────────────────────────┐
│  Compile time                                           │
│                                                         │
│  obfstr!("secret")                                      │
│  ├─ random!(u32, "key", stringify!("secret"))           │
│  │   └─ entropy(concat!(file!(), line!(), column!()))   │
│  ├─ keystream::<LEN>(key)     ← XorShift RNG            │
│  ├─ obfuscate(plaintext, keystream)  ← XOR each byte    │
│  └─ static _SDATA: [u8; LEN] = ciphertext               │
│                                                         │
│  Runtime                                                │
│                                                         │
│  ├─ read_volatile(&_SDATA)    ← block constant folding  │
│  ├─ deobfuscate(ciphertext, keystream)  ← XOR again     │
│  └─ return &str (temporary)                             │
│                                                         │
│  • Plaintext never appears in .rodata or the binary     │
│  • No heap allocations                                  │
│  • no_std compatible                                    │
└─────────────────────────────────────────────────────────┘
```

---

## Platform Support

| Target | Status |
|--------|--------|
| All Rust targets (`no_std`) | ✅ Full support |

Requires Rust **1.85+** (edition 2024).

---

## Building

```sh
cargo build --release
```

For deterministic (reproducible) builds:

```sh
OBFANY_SEED="0xDEAD_BEEF" cargo build --release
```

---

## Testing

```sh
# Unit + doc tests
cargo test

# Check for warnings
cargo check
```

---

## Verifying Obfuscation (Assembly Inspection)

The `examples/inspect_asm.rs` example is designed for manual verification that
obfuscation works at the binary level — plaintext strings and direct symbol
references must not appear in the compiled output.

```sh
# Sanity check
cargo run --example inspect_asm

# Dump assembly
cargo rustc --release --example inspect_asm -- --emit asm -C "llvm-args=-x86-asm-syntax=intel"

# Search for plaintext (should return NOTHING)
Select-String -Path target/release/examples/inspect_asm.s -Pattern "Hello world"
```

---

## Credits

Forked from [obfstr](https://github.com/CasualX/obfstr) by [CasualX](https://github.com/CasualX) (MIT).

Changes from upstream:
- Renamed crate to `obfany`
- Added `obfnum!` for numeric constant obfuscation
- Added compiletime key-collision detection in `obfstmt!`
- Replaced `transmute` with `from_bits` / `to_ne_bytes` (Rust 1.85+)
- Added SAFETY comments to all unsafe blocks
- Expanded test coverage (26 unit + 15 doc tests)

---

## License

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.

