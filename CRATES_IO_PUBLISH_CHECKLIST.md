# Crates.io Publishing Checklist

## Pre-release Preparation

### 1. Cargo.toml Configuration
- [x] Package name: `dlsite-gamebox` (to avoid conflict with original `dlsite`)
- [x] Version: `0.2.0`
- [x] Description: "High-performance DLsite client with caching, parallel parsing, and streaming support"
- [x] License: MIT
- [x] Repository: https://github.com/SuperToolman/dlsite-gamebox
- [x] Authors: SuperToolman
- [x] Keywords: dlsite, scraper, api, async, performance
- [x] Categories: api-bindings

### 2. Code Quality
- [x] All tests passing (27/27 unit tests)
- [x] No compilation errors
- [x] Documentation complete
- [x] Examples clear

### 3. Documentation
- [x] README.md complete
- [x] CHANGELOG.md complete
- [x] Code comments adequate
- [x] Example code runnable

### 4. License
- [x] LICENSE file exists
- [x] MIT license specified in Cargo.toml

### 5. Dependencies
- [x] All dependencies are public
- [x] No local path dependencies
- [x] Dependency versions reasonable

## Publishing Steps

### Step 1: Verify Package
```bash
cargo publish --dry-run
```

### Step 2: Commit Changes
```bash
git add Cargo.toml
git commit -m "chore: Update package name to dlsite-gamebox for crates.io"
git push origin master
```

### Step 3: Publish to crates.io
```bash
cargo publish
```

### Step 4: Verify Publication
Visit: https://crates.io/crates/dlsite-gamebox

## Pre-release Verification

### Check Cargo.toml
```toml
[package]
name = "dlsite-gamebox"
version = "0.2.0"
edition = "2021"
description = "High-performance DLsite client with caching, parallel parsing, and streaming support"
license = "MIT"
repository = "https://github.com/SuperToolman/dlsite-gamebox"
authors = ["SuperToolman"]
keywords = ["dlsite", "scraper", "api", "async", "performance"]
categories = ["api-bindings"]
```

### Check Files
- [x] Cargo.toml - Configuration correct
- [x] Cargo.lock - Exists (optional)
- [x] src/lib.rs - Public API correct
- [x] README.md - Contains usage examples
- [x] LICENSE - MIT license

## Package Information

| Item | Value |
|------|-------|
| Package name | dlsite-gamebox |
| Version | 0.2.0 |
| License | MIT |
| Repository | https://github.com/SuperToolman/dlsite-gamebox |
| Category | api-bindings |
| Keywords | dlsite, scraper, api, async, performance |

## Post-release

### 1. Create GitHub Release
```bash
git tag -a v0.2.0-crates -m "Release v0.2.0 to crates.io"
git push origin v0.2.0-crates
```

### 2. Update Documentation
- Add crates.io link to README
- Create Release notes on GitHub

### 3. Promotion
- Share in Rust community
- Update related documentation

## Important Notes

1. **Package name conflict**: Changed to `dlsite-gamebox` to avoid conflict with original `dlsite`
2. **Version number**: Starting from 0.2.0, indicating an optimized version
3. **Backward compatibility**: All APIs are backward compatible
4. **Documentation**: Ensure all public APIs have documentation

## Quick Publish Commands

```bash
# 1. Verify package (recommended to run first)
cargo publish --dry-run

# 2. Publish to crates.io
cargo publish

# 3. Verify successful publication
curl https://crates.io/api/v1/crates/dlsite-gamebox
```

## Post-publication Updates

After publishing, if updates are needed:
1. Modify version number in Cargo.toml
2. Update CHANGELOG.md
3. Commit and push to GitHub
4. Run `cargo publish`

## Done

Ready to publish! Run the following command:

```bash
cargo publish
```
