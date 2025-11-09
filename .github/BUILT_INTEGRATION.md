# Built Crate Integration

This project uses the [`built`](https://crates.io/crates/built) crate to embed build-time information into the binary.

## What it Provides

The `built` crate generates a `built.rs` file at compile time containing:
- Git commit hash (when built from a git repository)
- Build timestamp
- Rust version used
- Enabled features
- And more...

## Implementation

### 1. Build Dependency

```toml
[build-dependencies]
built = { version = "0.8.0", features = ["git2"] }
```

The `git2` feature enables git information extraction.

### 2. Build Script (`build.rs`)

```rust
fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
}
```

This generates `target/{profile}/build/cron-when-*/out/built.rs` at compile time.

### 3. Usage in CLI (`src/cli/commands/mod.rs`)

```rust
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn new() -> Command {
    let git_hash = built_info::GIT_COMMIT_HASH.unwrap_or("unknown");
    let long_version: &'static str =
        Box::leak(format!("{} - {}", env!("CARGO_PKG_VERSION"), git_hash).into_boxed_str());

    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_version(long_version)
        // ...
}
```

## Version Output Behavior

### When built from git repository:
```bash
$ cron-when --version
cron-when 0.1.0 - 5bb64edb296d182b2d2d89d4b3915e374a573bc3
```

Shows version + full git commit hash.

### When installed via `cargo install`:
```bash
$ cron-when --version
cron-when 0.1.0 - unknown
```

Shows version + "unknown" (no git repository available).

### When downloaded from GitHub release:
```bash
$ cron-when --version
cron-when 0.1.0 - 5bb64ed
```

Shows version + short commit hash (if built with git info).

## Why This Matters

### For Development
- Easily identify which commit a binary was built from
- Debug issues by knowing exact source version
- Track builds across environments

### For Users
- Verify they're running the correct version
- Report issues with exact build information
- Ensure releases match source code

### For CI/CD
- GitHub release workflow builds include commit hash
- RPM/DEB packages can include build metadata
- Reproducible builds can be verified

## Template Usage

This pattern works in any Rust project:

1. Add to `Cargo.toml`:
   ```toml
   [build-dependencies]
   built = { version = "0.8", features = ["git2"] }
   ```

2. Create `build.rs`:
   ```rust
   fn main() {
       built::write_built_file().expect("Failed to acquire build info");
   }
   ```

3. Use in your code:
   ```rust
   pub mod built_info {
       include!(concat!(env!("OUT_DIR"), "/built.rs"));
   }

   println!("Built from: {}",
       built_info::GIT_COMMIT_HASH.unwrap_or("unknown"));
   ```

4. Add to `.gitignore`:
   ```
   built.rs
   ```

## Available Information

The `built_info` module provides:

```rust
// Git information
GIT_COMMIT_HASH: Option<&str>       // Full commit hash
GIT_COMMIT_HASH_SHORT: Option<&str> // Short hash (7 chars)
GIT_DIRTY: Option<bool>             // Uncommitted changes?
GIT_HEAD_REF: Option<&str>          // Branch or tag name

// Build information
BUILT_TIME_UTC: &str                // RFC 2822 timestamp
RUSTC_VERSION: &str                 // Rust compiler version
PROFILE: &str                       // "debug" or "release"
FEATURES: &[&str]                   // Enabled cargo features

// Package information
PKG_VERSION: &str                   // From Cargo.toml
PKG_NAME: &str                      // Package name
PKG_AUTHORS: &str                   // Authors
```

## Best Practices

1. **Always use `.unwrap_or()`** for optional fields:
   ```rust
   built_info::GIT_COMMIT_HASH.unwrap_or("unknown")
   ```

2. **Use `Box::leak()` for static strings** when needed:
   ```rust
   let version: &'static str =
       Box::leak(format!("v{}", built_info::PKG_VERSION).into_boxed_str());
   ```

3. **Add build.rs to .gitignore** (the output, not the script):
   ```
   # Don't commit generated file
   built.rs
   ```

4. **Consider adding build info to error reports**:
   ```rust
   eprintln!("Version: {}", built_info::PKG_VERSION);
   eprintln!("Commit: {}", built_info::GIT_COMMIT_HASH.unwrap_or("unknown"));
   eprintln!("Built: {}", built_info::BUILT_TIME_UTC);
   ```

## Troubleshooting

### "failed to acquire build-time information"

**Cause:** `built` crate not in `[build-dependencies]`
**Fix:** Move from `[dependencies]` or `[dev-dependencies]` to `[build-dependencies]`

### "cannot find module `built_info`"

**Cause:** The `include!()` macro can't find the generated file
**Fix:** Ensure `build.rs` runs successfully:
```bash
cargo clean
cargo build
```

### "GIT_COMMIT_HASH is None"

**Cause:** Not building from a git repository
**Expected:** This is normal for `cargo install` or when building from tarballs
**Handle:** Always use `.unwrap_or("unknown")` or similar fallback

## References

- [built crate documentation](https://docs.rs/built)
- [built crate repository](https://github.com/lukaslueg/built)
- [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
