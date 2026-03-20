---
name: dlsite-rs-next codebase patterns
description: Naming conventions, locale/site validation approach, test structure, and URL utility patterns for dlsite-rs-next (dlsite-gamebox crate)
type: project
---

Crate name on crates.io is `dlsite-gamebox` (package name), but the repo is `dlsite-rs-next`.

**Why:** Renamed for crates.io publishing (commit 74968c5).
**How to apply:** Use `dlsite-gamebox` when referring to the published crate; use `dlsite-rs-next` for the repo path.

## Naming Conventions

- Path builders: implemented as `.to_path()` methods on query structs (`SearchProductQuery`, `CircleQuery`), not standalone `build{Resource}Path` functions.
- Sub-client pattern: `DlsiteClient.product()`, `.search()`, `.circle()`, `.ranking()` return short-lived sub-clients holding a `&DlsiteClient` reference.
- Locale-aware variants: `get_review_with_locale(...)` and `get_with_locale(...)` — the base method delegates to the locale variant with a default.

## URL Construction

- Macros in `src/client/search/macros.rs`: `push!`, `push_option!`, `push_option_array!`, `push_option_bool!`
- `push_option_bool!` emits `/field/1` for `Some(true)`, nothing for `Some(false)` or `None`
- `campaign` is special: uses inline `if let Some(true)` to emit `/campaign/campaign` (not `/campaign/1`)
- Search path starts with `/fsr/ajax/=`, circle path starts with `/circle/profile/=`
- Product API path: `/api/=/product.json?workno={id}&locale={locale}`
- Review API path: `/api/review?product_id=...&locale={locale}` where locale is BCP-47 form (`ja_JP`, `en_US`, etc.)

## Locale/Site Approach

- `Language` enum in `src/interface/query.rs` with strum `serialize_all = "snake_case"` for Display (path segment)
- `Language::to_review_locale()` returns BCP-47 strings for the review/API locale param
- `Site` enum in `src/interface/site.rs` with `base_url()` returning full URL
- Locale validation: enum-based, not passthrough. Only 5 variants: `Jp`, `En`, `Ko`, `ZhCn`, `ZhTw`.
- Default locale: `Language::Jp`; default site: `Site::Maniax`

## Test Structure

- Tests colocated in the same file as the implementation (`#[cfg(test)] mod tests { ... }`)
- Separate test files exist in `src/client/product/test.rs` and `src/client/product_api/test.rs`
- Test framework: Rust built-in + `tokio::test` for async, `test-case` crate for parameterized
- Live network tests are NOT marked `#[ignore]` — they hit real DLsite and may fail on HTML changes
- 3 pre-existing failing network tests: `search_product_1`, `search_product_2`, `get_circle_1` (thumbnail selector broken against live HTML)

## Known Gaps

- `rate_count` in `SearchProductItem` uses same selector as `dl_count` — likely copy-paste bug, needs live verification
- `parse_search_html` and `parse_search_html_parallel` have duplicated logic
- `get_raw` bypasses rate limiting and caching
- HTML scraping (`get_html`) still uses Japanese page regardless of locale
- Ranking, auth, play, user modules are stubs — endpoint URLs not verified via network capture
- Circle sale list: not implemented, needs network capture
