# DLsite Verified Capability Notes (Revised v6 - Implementation Complete)

Date checked: 2026-03-22
Base stance: All four documented capabilities are now implemented in code.

## Status summary

| Capability | Status | Implementation |
|---|---|---|
| `circle_name -> maker_id` | **Implemented** | `CircleClient::resolve_circle_name()` |
| `maker_id -> all games by circle` | **Implemented** | `CircleClient::list_circle_games()` |
| `product_id -> thumbnail` | **Implemented** | `ProductApiClient::get_product_thumbnail()` |
| `product_id -> screenshots` | **Implemented** | `ProductApiClient::list_product_screenshots()` |

---

## 1) Capability: `circle_name -> maker_id`

### Implementation
- **Method**: `CircleClient::resolve_circle_name(circle_name: &str) -> Result<Option<String>>`
- **File**: `src/client/circle/mod.rs`
- **Feature flag**: `search-html`

### Endpoint used
- `/home/circle/list/=/name_header/{bucket}`
- Purpose: List route used to exact-match a circle name inside one bucket.

### Implementation details
- `get_name_bucket()` determines the bucket from the first character (hiragana/katakana/alphanumeric).
- Fetches the bucket page and parses HTML for circle links.
- Exact-matches the circle name and extracts maker_id from the profile URL.

### Caveats
- Returns `None` if no exact match is found.
- Requires `search-html` feature flag.

---

## 2) Capability: `maker_id -> all games by circle`

### Implementation
- **Method**: `CircleClient::list_circle_games(maker_id: &str) -> Result<Vec<SearchProductItem>>`
- **File**: `src/client/circle/mod.rs`
- **Feature flag**: `search-html`

### Endpoint used
- `/{floor}/circle/profile/=/maker_id/{maker_id}.html`
- Purpose: Primary dataset for enumerating a circle's works.

### Implementation details
- Reuses `get_circle()` to fetch the circle's product list.
- Filters products using `WorkType::is_game()` helper method.
- Game types: ACN, QIZ, ADV, RPG, TBL, DNV, SLN, TYP, STG, PZL, ETC.

### Caveats
- Uses the default floor (home). For cross-floor enumeration, call `get_circle()` directly with custom options.

---

## 3) Capability: `product_id -> thumbnail`

### Implementation
- **Method**: `ProductApiClient::get_product_thumbnail(id: &str) -> Result<String>`
- **File**: `src/client/product_api/mod.rs`
- **Feature flag**: None (uses JSON API)

### Endpoint used
- `/api/=/product.json?workno={id}&locale={locale}`
- Purpose: JSON API that returns `image_thum.url` field.

### Implementation details
- Calls existing `get()` method and extracts `product.image_thum.url`.
- Simple wrapper over the existing product API.

---

## 4) Capability: `product_id -> screenshots`

### Implementation
- **Method**: `ProductApiClient::list_product_screenshots(id: &str) -> Result<Vec<String>>`
- **File**: `src/client/product_api/mod.rs`
- **Feature flag**: None (uses JSON API)

### Endpoint used
- `/api/=/product.json?workno={id}&locale={locale}`
- Purpose: JSON API that returns `image_samples` field.

### Implementation details
- Calls existing `get()` method and extracts URLs from `product.image_samples`.
- Returns empty `Vec` if no samples are available.

---

## Helper: `WorkType::is_game()`

### Implementation
- **Method**: `WorkType::is_game(&self) -> bool`
- **File**: `src/interface/product.rs`

### Game work types
- ACN (Action), QIZ (Quiz), ADV (Adventure), RPG (Role-playing)
- TBL (Table), DNV (Digital Novel), SLN (Simulation)
- TYP (Typing), STG (Shooting), PZL (Puzzle), ETC (Other Games)

---

## API Summary

```rust
// Product API client (no feature flag required)
let client = DlsiteClient::default();

// Get thumbnail URL
let thumbnail_url = client.product_api().get_product_thumbnail("RJ01014447").await?;

// Get screenshot URLs
let screenshots = client.product_api().list_product_screenshots("RJ01014447").await?;

// Circle client (requires search-html feature)
let client = DlsiteClient::default();

// List all games from a circle
let games = client.circle().list_circle_games("RG24350").await?;

// Resolve circle name to maker ID
let maker_id = client.circle().resolve_circle_name("circle name").await?;

// Check if a work type is a game
use dlsite_gamebox::interface::product::WorkType;
assert!(WorkType::RPG.is_game());
assert!(!WorkType::MNG.is_game());
```
