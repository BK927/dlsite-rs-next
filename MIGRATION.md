# Migration Guide

This document describes the breaking changes introduced in version 0.2.0 and provides guidance for migrating from the previous version.

## Summary of Changes

### 1. HTML Scraping Removed from Product Module

The `get_html()` method and the `html` module have been removed from the `ProductClient`. All product information is now obtained exclusively through JSON APIs.

### 2. Feature Flag Requirements

The following features now require explicit feature flags:

| Feature | Required Flag | Dependencies Added |
|---------|---------------|-------------------|
| Search (`search_product`, etc.) | `search-html` | `scraper`, `rayon` |
| Circle profile (`get_circle_profile`) | `search-html` | `scraper`, `rayon` |
| Play API (library, downloads, viewer) | `cookie-store` | `reqwest/cookies` |

### 3. Type Changes

| Type | Before | After |
|------|--------|-------|
| `Product.released_at` | `NaiveDate` | `Option<NaiveDate>` |
| `Product.circle_id` | `String` | `Option<String>` |
| `Product.circle_name` | `String` | `Option<String>` |
| `Product.people` | `ProductPeople` | `Option<ProductPeople>` |

### 4. Removed Fields

The following fields have been removed from `Product` as they are not available in JSON APIs:

| Field | Type | Reason |
|-------|------|--------|
| `file_format` | `Vec<String>` | Not available in JSON APIs |
| `product_format` | `Vec<String>` | Not available in JSON APIs |

## Migration Steps

### Step 1: Update Cargo.toml

If you use search or circle features, add the `search-html` feature flag:

```toml
[dependencies]
dlsite-gamebox = { version = "0.2", features = ["search-html"] }
```

If you use Play API features, add the `cookie-store` feature flag:

```toml
[dependencies]
dlsite-gamebox = { version = "0.2", features = ["cookie-store"] }
```

### Step 2: Update Product Usage

**Before:**
```rust
let product = client.product().get_all("RJ123456").await?;

// These were always present
let id = &product.circle_id;    // String
let name = &product.circle_name; // String
let date = product.released_at;  // NaiveDate
let people = product.people;     // ProductPeople
```

**After:**
```rust
let product = client.product().get_all("RJ123456").await?;

// These are now optional
if let Some(id) = &product.circle_id {
    // ...
}
if let Some(name) = &product.circle_name {
    // ...
}
if let Some(date) = product.released_at {
    // ...
}
if let Some(people) = &product.people {
    // Access people.author, people.voice_actor, etc.
}
```

### Step 3: Handle Removed Fields

The `file_format` and `product_format` fields have been removed. If you need this information:

1. **File format**: Not available via JSON APIs. Consider using the `ProductApiContent.file_type` field from `product_api().get()` instead.
2. **Product format**: Not available via JSON APIs. Consider using the `ProductApiContent.work_type` field from `product_api().get()` instead.

### Step 4: Feature-Gated Code

If you use search or circle features, wrap your code in feature gates:

```rust
#[cfg(feature = "search-html")]
{
    let results = client.search().search_product(&query).await?;
    // ...
}
```

Or add compile-time error messages:

```rust
#[cfg(not(feature = "search-html"))]
compile_error!("Search functionality requires the 'search-html' feature flag. Add `features = [\"search-html\"]` to your Cargo.toml.");
```

## New Features

### Play v3 API Adapters

With the `cookie-store` feature, you can now access:

1. **Library Access**: View purchased works
   ```rust
   let count = client.play().library().get_count(None).await?;
   let sales = client.play().library().get_sales(None).await?;
   ```

2. **Download URLs**: Get download targets for purchased works
   ```rust
   let target = client.play().download().get_download_target("RJ123456").await?;
   if let Some(url) = target.url {
      // Use the download URL
   }
   ```

3. **Viewer Sessions**: Access streaming content
   ```rust
   let token = client.play().viewer().get_manifest_token("RJ123456").await?;
   ```

### New Error Types

The error enum has been expanded:

- `WorkNotFound(String)`: Work ID was not found
- `DownloadNotAvailable(String)`: Download is not available for this work
- `PlayNotAvailable(String)`: DLsite Play is not available for this work
- `FeatureGated { feature, required_flag }`: Feature requires a specific flag

## Benefits of This Refactoring

1. **More Stable**: No more HTML scraping means the library won't break when DLsite changes their page structure
2. **Smaller Dependency Tree**: Users who don't need search can avoid the `scraper` and `rayon` dependencies
3. **Better Auth Support**: New adapters provide a foundation for authenticated access to library and downloads
4. **Clearer API Surface**: Feature flags make it explicit which features require which dependencies

## Getting Help

If you encounter issues migrating:

1. Check the [GitHub Issues](https://github.com/SuperToolman/dlsite-gamebox/issues)
2. Provide a minimal reproduction case
3. Include the version you're migrating from and to
