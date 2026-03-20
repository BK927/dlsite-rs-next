# DLsite Endpoint Inventory

Matrix of all implemented and known-but-unimplemented endpoints in this library.
Confidence levels: high (tested against live data), medium (URL known, not verified), low (inferred from HTML/docs).

---

## Implemented Endpoints

| Module | Endpoint Path | Auth Required? | Request Params | Response Shape | Status | Missing Params | Confidence |
|--------|---------------|---------------|----------------|----------------|--------|----------------|------------|
| `product` | `/work/=/product_id/{id}` | No | `product_id` (path) | `ProductHtml` (scraped) | Implemented | none | High |
| `product` | `/product/info/ajax?product_id={id}` | No | `product_id` (query) | `ProductAjax` (JSON) | Implemented | none | High |
| `product` | `/api/review?product_id=...` | No | `product_id`, `limit`, `page`, `mix_pickup`, `order`, `locale` | `ProductReview` (JSON) | Implemented | none after Phase 3 | High |
| `product_api` | `/api/=/product.json?workno={id}&locale={locale}` | No | `workno`, `locale` | `ProductApiContent` (JSON) | Implemented | none after Phase 3 | Medium |
| `search` | `/fsr/ajax/=/{params}` | No | See `SearchProductQuery` | `SearchAjaxResult` → `Vec<SearchProductItem>` | Implemented | none after Phase 2 | High |
| `circle` | `/circle/profile/=/{params}` | No | `circle_id`, `order`, `per_page`, `page` | `SearchResult` (scraped) | Implemented | options array not verified | Medium |
| `circle` | (profile metadata) | No | `circle_id` | `CircleProfile` | Implemented (Phase 6) | full field list unverified | Low |

---

## Known-but-Unimplemented Endpoints

| Module | Endpoint Path | Auth Required? | Notes |
|--------|---------------|---------------|-------|
| `ranking` | Unknown — needs network capture | Unknown | DLsite has trending/bestseller pages; exact API path not confirmed |
| `circle` | Circle sale list | Likely No | HTML path inferred; not captured |
| `auth` | Login/session endpoints | Yes | Requires cookie-store; stubs only in Phase 7 |
| `play` | DLsite Play streaming | Yes | Feature-gated; stubs only in Phase 7 |
| `user` | User library/purchases | Yes | Feature-gated; stubs only in Phase 7 |

---

## Search Parameter Coverage (after Phase 2)

`SearchProductQuery` now covers all parameters listed in the URL comment at the top of `query.rs`:

| Parameter | Type | Implemented |
|-----------|------|-------------|
| `language` | `Language` | Yes |
| `keyword_creator` | `Option<String>` | Yes |
| `sex_category` | `Option<Vec<SexCategory>>` | Yes |
| `keyword` | `Option<String>` | Yes |
| `regist_date_end` | `Option<String>` | Yes |
| `regist_date_start` | `Option<String>` | Yes (added Phase 2) |
| `price_low` | `Option<u32>` | Yes |
| `price_high` | `Option<u32>` | Yes |
| `ana_flg` | `Option<AnaFlg>` | Yes |
| `age_category` | `Option<Vec<AgeCategory>>` | Yes |
| `work_category` | `Option<Vec<WorkCategory>>` | Yes |
| `order` | `Option<Order>` | Yes |
| `work_type` | `Option<Vec<WorkType>>` | Yes |
| `work_type_category` | `Option<Vec<WorkTypeCategory>>` | Yes |
| `work_type_category_name` | `Option<Vec<String>>` | Yes (added Phase 2) |
| `genre` | `Option<Vec<u32>>` | Yes |
| `genre_name` | `Option<Vec<String>>` | Yes (added Phase 2) |
| `options_and_or` | `Option<OptionAndOr>` | Yes |
| `options` | `Option<Vec<String>>` | Yes |
| `options_not` | `Option<Vec<String>>` | Yes |
| `options_name` | `Option<Vec<String>>` | Yes (added Phase 2) |
| `file_type` | `Option<Vec<FileType>>` | Yes |
| `rate_average` | `Option<u32>` | Yes |
| `per_page` | `Option<u32>` | Yes |
| `page` | `Option<u32>` | Yes |
| `campaign` | `Option<bool>` | Yes (fixed typo Phase 2) |
| `soon` | `Option<bool>` | Yes |
| `dlsite_only` | `Option<bool>` | Yes (added Phase 2) |
| `is_pointup` | `Option<bool>` | Yes |
| `is_free` | `Option<bool>` | Yes |
| `release_term` | `Option<ReleaseTerm>` | Yes |
| `price_category` | `Option<u32>` | Yes (added Phase 2) |
| `show_type` | `Option<u32>` | Yes (added Phase 2) |
| `from` | `Option<String>` | Yes (added Phase 2) |
