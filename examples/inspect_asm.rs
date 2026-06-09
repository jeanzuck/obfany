/*!
Inspect the generated assembly to verify obfuscation works at the binary level.

# Step 1 — Run (sanity check)
  cargo run --example inspect_asm
  Expected output:
    obfstmt: 69016
    obfstr: Hello world!
    obfstr: AB
    obfstr: This literal is very very very long to see if it correctly handles long strings
    xref: 3141592

# Step 2 — Compile + dump assembly
  cargo rustc --release --example inspect_asm -- --emit asm -C "llvm-args=-x86-asm-syntax=intel"
  → Assembly written to target/release/examples/inspect_asm.s

# Step 3 — Verify the assembly

## 3a. obfstr — plaintext must NOT appear in .rodata
  In the assembly file, search for each obfuscated string:
    - "Hello world!"  → should NOT be found
    - "AB"            → should NOT be found
    - "This literal…" → should NOT be found
  If any plaintext appears as a static string constant, the obfuscation failed.

## 3b. obfstmt — control flow must be randomised
  Find the `obfstmt` function in the assembly.
  You should see a `match`-like dispatch (cmp/jump table) where each branch
  corresponds to one statement — but the order is not the source order.
  The original sequence `i = 5; i *= 24; …` should not be visible as a
  straight-line block.

## 3c. xref — no direct symbol reference
  Find the `xref` function. It should NOT contain a direct `lea` instruction
  pointing to the static `FOO`. Instead you should see opaque pointer arithmetic
  (add/sub/xor/rotate) that reconstructs the address at runtime.

# How #[inline(never)] helps
  Each function is marked #[inline(never)] so it appears as a standalone
  symbol in the assembly. In a real binary these functions would be inlined
  and mixed with surrounding code — the obfuscation still works, but it is
  harder to isolate for inspection.
*/

#[inline(never)]
fn obfstmt() -> i32 {
    let mut i = 0;
    // trace_macros!(true);
    obfany::obfstmt! {
        i = 5;
        i *= 24;
        i -= 10;
        i += 8;
        i *= 28;
        i -= 18;
        i += 1;
        i *= 21;
        i -= 11;
    }
    // trace_macros!(false);
    assert_eq!(i, 69016);
    i
}

#[inline(never)]
fn obfstr() {
    print(obfany::obfstr!("Hello world!"));
    print(obfany::obfstr!("AB"));
    print(obfany::obfstr!(
        "This literal is very very very long to see if it correctly handles long strings"
    ));
}

#[inline(never)]
fn xref() -> &'static i32 {
    static FOO: i32 = 3141592;
    obfany::xref!(&FOO)
}

fn main() {
    println!("obfstmt: {}", obfstmt());
    obfstr();
    println!("xref: {}", xref());
}

#[inline(never)]
fn print(s: &str) {
    println!("obfstr: {}", s);
}
