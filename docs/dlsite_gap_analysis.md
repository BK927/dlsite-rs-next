# DLsite Library Gap Analysis

Categorizes all features by implementation state as of v0.3.0.

---

## 1. Implemented (ready for production use)

- Product HTML scraping (`ProductClient::get_html`)
- Product AJAX details (`ProductClient::get_ajax`, `get_ajax_multiple`)
- Product review fetch (`ProductClient::get_review`, `get_review_with_locale`)
- Product API JSON fetch (`ProductApiClient::get`, `get_with_locale`)
- Search with full parameter set (`SearchClient::search_product`, `search_products_batch`, `search_product_stream`)
- Circle product listing (`CircleClient::get_circle`)
- Circle profile metadata (`CircleClient::get_circle_profile`)
- Response caching (LRU, configurable TTL)
- Rate limiting (500ms between requests)
- Retry with exponential backoff
- Error taxonomy: `AuthRequired`, `SessionExpired`, `SchemaDrift`
- Locale support: `Language` enum with `Jp`, `En`, `Ko`, `ZhCn`, `ZhTw`
- Site abstraction: `Site` enum with `base_url()`

---

## 2. Partially Implemented

- **Multi-language support**: `Language` enum now has 5 variants; `get_review_with_locale` and `get_with_locale` accept them. However, the HTML scraping pipeline (`get_html`) still fetches the Japanese page only — locale is not applied to the scrape path.
- **Circle profile**: Implemented as a scrape of the same circle HTML page. Field coverage (`CircleProfile`) may be incomplete — only `id`, `name`, `description`, `banner_url` are captured. Requires live verification.

---

## 3. Documented-but-Absent (in code comments / README, not yet implemented)

- **Circle sale list**: Mentioned in README and `CircleClient` docs. The endpoint path has not been captured from network traffic. Do not implement without a verified URL.
- **Ranking**: Mentioned in README. No endpoint URL confirmed. `RankingClient` exists as a stub with TODO comments only.
- **Login / Auth**: Referenced in README. `AuthClient` stub exists behind `cookie-store` feature gate. No actual login flow implemented.
- **DLsite Play streaming**: `PlayClient` stub exists behind `cookie-store` feature gate. Not implemented.
- **User library / purchases**: `UserClient` stub behind `cookie-store` feature gate. Not implemented.

---

## 4. Needs Network Capture Before Implementation

The following cannot be correctly implemented without capturing actual browser/app traffic:

| Feature | Reason |
|---------|--------|
| Ranking endpoint | Exact API path unknown; HTML structure unverified |
| Circle sale list | Endpoint URL not confirmed |
| Auth/login flow | Session cookie behavior unknown; CSRF token handling unknown |
| Play streaming | Protocol (HLS? proprietary?) unknown |

---

## 5. Bugs Found and Fixed (in this release)

| Bug | Location | Fix Applied |
|-----|----------|-------------|
| `campagin` typo | `search/query.rs` field name | Renamed to `campaign`; path generation fixed to `campaign/campaign` |
| Duplicate `per_page` | `circle/query.rs` lines 20-21 | Removed duplicate `push_option!` call |
| Hardcoded `locale=ja_JP` | `product/mod.rs::get_review` | Replaced with `locale.to_review_locale()` via new `get_review_with_locale` |
| Stale GitHub URL | `product_api/mod.rs` | `ozonezone/dlsite-rs` → `SuperToolman/dlsite-gamebox` |
| Dead README link | `README.md` | Removed link to `QUERY_PERFORMANCE_OPTIMIZATION.md` |
| Missing search params | `search/query.rs` | Added 8 params: `regist_date_start`, `genre_name`, `options_name`, `work_type_category_name`, `show_type`, `from`, `dlsite_only`, `price_category` |

---

## 6. Out-of-Scope Observations (not touched)

- The `get_raw` method bypasses rate limiting and caching — could be a footgun for callers who use it for high-volume requests.
- `parse_search_html` and `parse_search_html_parallel` are both public (`pub(crate)`) with nearly identical logic duplicated across them. Deduplication is warranted but out of scope here.
- The `rate_count` field in `SearchProductItem` uses the same selector as `dl_count`, which appears to be a copy-paste bug in the original code. Not fixed here as it requires live verification of the correct selector.
- The `review::ProductReview` struct derives `Deserialize` from `serde_json::Value` in `get_review` but `is_success` is checked on the raw JSON before deserialization — if the field is missing, the error message is misleading. This is a pre-existing issue.
