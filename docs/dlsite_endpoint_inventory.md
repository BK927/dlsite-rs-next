# DLsite Capability Notes

## 1. 문서 목적

이 문서는 기존의 웹 실측 중심 DLsite capability 문서를, 실제 저장소 코드 구현 기준으로 다시 정리한 버전이다.  
따라서 이 문서는 “현재 저장소가 실제로 어떤 DLsite 기능을 구현하고 있는가”에 초점을 둔다.

---

## 2. 상태 요약

| Capability | Status | 현재 저장소 구현 여부 | 이번 문서 처리 |
|---|---|---:|---|
| `product_id (RJ) -> direct metadata fetch` | **Implemented** | 예 | 추가 |
| `product_id -> official titles (ko/ja/en)` | **Implemented** | 예 | 추가 |
| `product_id -> tags` | **Implemented** | 예 | 추가 |
| `product_id -> developer / publisher / release_year` | **Implemented** | 예 | 추가 |
| `product_id -> thumbnail` | **Implemented** | 예 | 기존 내용 수정/보강 |
| `product_id -> screenshots` | **Implemented** | 예 | 기존 내용 수정/보강 |
| `product_id -> rating / review_count / star_counts` | **Implemented** | 예 | 추가 |
| `title -> candidate RJ list` | **Implemented** | 예 | 추가 |
| `title -> best-match RJ metadata` | **Implemented** | 예 | 추가 |
| `mixed hint normalization (e.g. RJ + STEAM)` | **Implemented** | 예 | 추가 |
| `browser fallback search` | **Supported fallback** | 예 | 추가 |
| `circle_name -> maker_id` | **Implemented** | 예 | 추가 |
| `maker_id -> all games by circle` | **Implemented** | 예 | 추가 |

---

## 3. 엔드포인트별 정리

## 3.1 `/maniax/api/=/product.json?workno={RJ}&locale={locale}`

### Status
- **Implemented**
- 지원 locale: `ko_KR`, `ja_JP`, `en_US`, `zh_CN`, `zh_TW`

### 이 엔드포인트로 구현된 기능
- `product_id -> official titles (ko/ja/en)`
- `product_id -> tags input`
- `product_id -> work_type / file_type / maker info`
- `product_id -> thumbnail candidates`
- `product_id -> screenshot candidates`
- `product_id -> rating fields`
- `product_id -> release date / year input`

### 저장소 동작
저장소는 같은 RJ 코드에 대해 아래 5개 locale 요청을 각각 보낸다.

- `...product.json?workno={RJ}&locale=ko_KR`
- `...product.json?workno={RJ}&locale=ja_JP`
- `...product.json?workno={RJ}&locale=en_US`
- `...product.json?workno={RJ}&locale=zh_CN`
- `...product.json?workno={RJ}&locale=zh_TW`

그 다음 locale별 row를 합쳐 `StoreMetadata`를 구성한다.

### 대표적으로 읽는 필드
- 제목: `work_name`, `name`
- 태그: `genres`
- 제작자/서클: `maker_name`, `maker_name_ja`, `maker_name_en`, `circle_name`, `brand`, `maker_id`
- 미디어: `image_main`, `work_image`, `main_img`, `main_image`, `image`, `image_url`
- 스크린샷 배열: `image_samples`, `sample_images`, `sample_image`, `images`, `work_images`, `work_image_list`
- 평점: `rate_average_2dp`, `rate_count`, `review_count`, `rate_count_detail`
- 날짜: `regist_date`, `release_date`, `work_date`, `sales_date`

### 결과 필드
- `official_title_ko`
- `official_title_ja`
- `official_title_en`
- `original_title`
- `developer`
- `publisher`
- `release_year`
- `tags`
- `thumbnail_url`
- `screenshots`
- `rating_value`
- `review_count`
- `rating_detail`

### Caveats
- 이 저장소는 locale별 태그를 따로 영구 저장하지 않는다.
- 대신 locale별 `genres`를 모은 뒤 병합해서 **단일 `tags: Vec<String>`** 로 저장한다.
- 즉 “언어별로 받아오느냐?”는 예, “언어별 배열로 저장하느냐?”는 아니오다.

---

## 3.2 `/maniax/product/info/ajax?cdn_cache_min=1&product_id={RJ}`

### Status
- **Implemented**
- `product.json` 보조 데이터 소스로 사용

### 이 엔드포인트로 보강되는 기능
- `product_id -> rating detail`
- `product_id -> thumbnail candidates`
- `product_id -> screenshots`
- 일부 maker / work metadata 보강

### 저장소 동작
저장소는 `product.json` 3개 locale 응답과 함께 `info/ajax`도 같이 조회한 뒤, `parse_dlsite_metadata()`에서 통합 파싱한다.

### 대표 보강 포인트
- `rate_count_detail`에서 별점 분포 추출
- `work_image`, `image_main` 류의 미디어 필드 보강
- `review_count`, `rate_count` 보강

### Caveats
- 실제 저장값은 “product.json 우선 + info/ajax 보강” 형태다.
- 따라서 단일 엔드포인트만으로 완전한 메타데이터를 구성하는 구조는 아니다.

---

## 3.3 `/maniax/fsr/ajax/=/language/jp/keyword/{query}`

### Status
- **Implemented**

### 이 엔드포인트로 구현된 기능
- `title -> candidate RJ list`
- `title -> best-match RJ metadata`의 1차 후보 수집

### 저장소 동작
저장소는 제목 검색 시 이 엔드포인트를 호출하고, 응답 JSON 안의 `search_result` HTML을 파싱한다.  
만약 JSON 파싱이 불충분하면 raw HTML 텍스트 파싱도 시도한다.

### 파싱 결과
- RJ 코드 추출
- 검색 결과 카드의 title 텍스트 추출
- 순서 보존
- 빈 검색 결과 탐지

### 내부 구현 메모
- `<li data-list_item_product_id="RJ...">` 패턴을 기준으로 후보를 읽는다.
- `work_name` 영역에서 title attribute 또는 anchor text를 읽는다.
- 결과가 완전히 비었는지 `work_not_found` 문구도 검사한다.

### Caveats
- 이 저장소는 검색 후보를 가져온 뒤, 각 RJ에 대해 다시 `product.json + info/ajax`를 조회해서 최종 후보 품질을 판단한다.
- 즉 검색 엔드포인트는 “후보 수집” 역할이지, 최종 메타데이터 확정 엔드포인트는 아니다.

---

## 3.4 Browser fallback (`run_drissionpage_fallback`)

### Status
- **Supported fallback**

### 목적
- `/fsr/ajax` 기반 검색이 실패하거나 결과가 비었을 때, 브라우저 자동화를 통해 RJ 코드를 회수하기 위한 보조 경로

### 저장소 동작
- Python fallback script가 존재하면 `uv run ... store_fallback.py --term {title}` 형태로 호출
- stdout JSON에서 `dlsiteRjCode` 또는 `dlsite_rj_code`를 읽음

### Caveats
- 기본 경로가 아니라 fallback 경로다.
- 문서상 “공식 API”보다는 “검색 복원용 보조 수단”으로 취급하는 것이 맞다.

---

## 3.5 `/api/review?product_id={RJ}&locale={locale}&limit={limit}&page={page}&order={order}&mix_pickup={bool}`

### Status
- **Implemented**

### 이 엔드포인트로 구현된 기능
- `product_id -> reviews`
- `product_id -> reviewer_genre_list`

### 저장소 동작
- locale 지원: `ja_JP`, `en_US`, `ko_KR`, `zh_CN`, `zh_TW`
- 정렬 옵션: `regist_d` (최신순), `top` (평점순)
- 페이지네이션: `limit`, `page` 파라미터로 제어

### 대표적으로 읽는 필드
- 리뷰 내용: `comment`, `review_text`
- 평점: `rating`, `rate`
- 작성자: `reviewer_name`, `nickname`
- 날짜: `regist_date`, `write_date`
- 추천수: `helpful_count`, `good_count`
- 성별/장르: `reviewer_gender`, `reviewer_genre_list`

### 사용 예시
- `ProductClient::get_review(product_id)` - 기본 리뷰 조회
- `ProductClient::get_review_with_locale(product_id, locale)` - 특정 locale 리뷰 조회

---

## 4. Capability별 정리

## 4.1 `product_id -> direct metadata fetch`

### Status
- **Implemented**

### Endpoint list
1. `/maniax/api/=/product.json?workno={RJ}&locale=ko_KR`
2. `/maniax/api/=/product.json?workno={RJ}&locale=ja_JP`
3. `/maniax/api/=/product.json?workno={RJ}&locale=en_US`
4. `/maniax/api/=/product.json?workno={RJ}&locale=zh_CN`
5. `/maniax/api/=/product.json?workno={RJ}&locale=zh_TW`
6. `/maniax/product/info/ajax?cdn_cache_min=1&product_id={RJ}`

### 구현 요약
- RJ 코드 하나로 다국어 메타데이터와 보조 info 데이터를 조회해 `StoreMetadata`를 구성한다.

### Keep / remove
- **Keep in core inventory**

---

## 4.2 `product_id -> official titles (ko/ja/en)`

### Status
- **Implemented**

### 구현 요약
- locale별 `product.json`에서 제목을 읽고,
- `official_title_ko`, `official_title_ja`, `official_title_en`, `original_title`를 채운다.

### Caveats
- 저장소는 locale별 title 필드는 따로 유지하지만,
- 어떤 언어가 최종 `title` 필드의 대표값이 될지는 우선순위 로직에 따라 정해진다.

### Keep / remove
- **Keep in core inventory**

---

## 4.3 `product_id -> tags`

### Status
- **Implemented**

### 구현 요약
- source는 `genres`를 우선 사용한다.
- `custom_genres` 필드도 파싱되어 있으며, 필요시 추가 태그 소스로 활용 가능하다.
- ko / ja / en / zh-cn / zh-tw / other row의 `genres`를 모아 동일 태그를 병합한다.
- 최종 표기 우선순위는 **ko > ja > en > zh-cn > zh-tw > other** 이다.
- 저장 형식은 `tags: Vec<String>` 단일 배열이다.

### 현재 문서 대비 추가된 포인트
- 기존 문서에는 tags 기능이 빠져 있었지만, 저장소에는 이미 구현돼 있다.
- 단, “언어별 태그를 각각 별도 배열로 반환”하는 구조는 아니다.

### Keep / remove
- **Keep as supported capability**
- **문서에 'locale-aware merge, single stored tag array'를 명시할 것**

---

## 4.4 `product_id -> developer / publisher / release_year`

### Status
- **Implemented**

### 구현 요약
- maker / circle / brand 관련 필드에서 개발자/퍼블리셔 성격 값을 읽는다.
- 날짜 텍스트에서 연도를 추출해 `release_year`를 채운다.

### Caveats
- DLsite 원천 데이터 특성상 `developer`와 `publisher`가 같은 값으로 채워질 수 있다.
- 이 저장소는 엄밀한 법인/개인 구분보다 메타데이터 보존을 우선한다.

### Keep / remove
- **Keep as supported capability**

---

## 4.5 `product_id -> thumbnail`

### Status
- **Implemented**

### 구현 방식
- 기존 문서처럼 canonical img_main 경로를 추정하는 방식이 아니라,
- JSON row의 미디어 필드(`image_main`, `work_image`, `main_img`, `main_image`, `image`, `image_url`)를 우선 읽어 `thumbnail_url`을 만든다.
- 상대경로나 `//` 경로는 정규화한다.

### 현재 문서 대비 수정 포인트
- 저장소 기준으로는 “표준 썸네일 경로 추정”보다 **JSON 필드 직접 사용**이 더 정확한 설명이다.

### Keep / remove
- **Keep in core inventory**
- **문구를 repo behavior 기준으로 수정할 것**

---

## 4.6 `product_id -> screenshots`

### Status
- **Implemented**

### 구현 방식
- `image_samples`, `sample_images`, `sample_image`, `images`, `work_images`, `work_image_list` 같은 배열 필드에서 스크린샷 URL들을 수집한다.
- 배열이 비어 있으면 `thumbnail_url`을 스크린샷 목록의 fallback으로 넣는다.

### 현재 문서 대비 수정 포인트
- 기존 문서는 HTML `作品内容`의 `Image:` 앵커 파싱 중심으로 썼다.
- 하지만 이 저장소 기준으로는 **JSON 배열 기반 수집**이 1차 경로다.

### Caveats
- 코드 기준으로는 “공식 별도 screenshot API”가 아니라 product/info JSON 계열 미디어 필드 활용에 가깝다.

### Keep / remove
- **Keep in core inventory**
- **설명을 HTML 중심에서 JSON 배열 중심으로 수정할 것**

---

## 4.7 `product_id -> rating / review_count / star_counts`

### Status
- **Implemented**

### 구현 요약
- `rate_average_2dp` -> `rating_value`
- `rate_count` / `review_count` -> `review_count` 및 `rating_label`
- `rate_count_detail` -> `star_counts` / `rating_detail`

### 추가로 하는 일
- 저장소는 이 정보를 `game_store_ratings`에도 저장할 수 있도록 구조를 갖고 있다.
- 이후 rating sort score 재계산에도 사용한다.

### 현재 문서 대비 추가된 포인트
- 기존 문서에는 평점/별점 분포 capability가 빠져 있었다.
- 저장소에는 이미 구현돼 있으므로 이번 문서에 포함한다.

### Keep / remove
- **Keep as supported capability**

---

## 4.8 `title -> candidate RJ list`

### Status
- **Implemented**

### Endpoint list
1. `/maniax/fsr/ajax/=/language/jp/keyword/{query}`
2. fallback: browser automation script

### 구현 요약
- query variants를 여러 개 생성한다.
- 각 query로 DLsite search endpoint를 호출한다.
- 검색 결과 카드에서 RJ 후보를 읽는다.
- 각 후보에 대해 다시 direct metadata fetch를 해 점수를 매긴다.

### Caveats
- 단순 문자열 검색이 아니라 query variant, dedupe, threshold가 들어간다.
- 즉 “검색 엔드포인트만 때려서 바로 끝”나는 구조는 아니다.

### Keep / remove
- **Keep in core inventory**

---

## 4.9 `title -> best-match RJ metadata`

### Status
- **Implemented**

### 구현 요약
- 검색 후보 수집 후,
- 각 RJ의 실제 metadata를 다시 조회하고,
- title similarity / strict similarity / precision scoring을 적용해 최종 후보를 고른다.

### Keep / remove
- **Keep in core inventory**

---

## 4.10 `mixed hint normalization`

### Status
- **Implemented**

### 구현 요약
- 예: `RJ411830+STEAM-9999999`, `RJ114100+STEAM-5538981`
- 문자열 안에 섞여 있는 RJ 코드를 정규식으로 먼저 추출하고 정규화한다.
- 이후 direct RJ lookup에 사용한다.

### Keep / remove
- **Keep as helper capability**

---

## 4.11 `work classification / non-game filtering`

### Status
- **Implemented**

### 구현 요약
- `work_type`, `file_type`, 설명 문자열을 보고 작품을 분류한다.
- 명시적으로 `ICG` 같은 CG/일러스트 work type은 제외한다.
- 반대로 RPG / ADV / SLN / action/adventure/simulation 계열 힌트는 게임으로 본다.

### Caveats
- 이 로직은 “게임만 남기기”라기보다 “명백히 비게임인 케이스를 제거”하는 보수적 필터에 가깝다.
- 테스트상 디지털노벨, 영상형 EXE 계열을 완전히 배제하지는 않는다.

### Keep / remove
- **Keep as implementation note**
- **외부 capability 문서에는 필터 정책으로 명시할 것**

---

## 4.12 `circle_name -> maker_id`

### Status
- **Implemented**

### 구현 요약
- `CircleClient::resolve_circle_name(circle_name)` - circle name을 검색하여 maker_id로 변환
- 내부적으로 DLsite 검색 API를 사용해 circle의 제품을 찾고, 그 중 하나에서 maker_id를 추출

### Endpoint list
1. `/maniax/fsr/ajax/=/language/jp/keyword/{circle_name}` - circle 이름으로 검색

### Caveats
- 간접 해결 방식이므로 정확도는 검색 결과 품질에 의존한다.
- 동명의 circle이 여러 개일 수 있어 주의가 필요하다.

### Keep / remove
- **Keep as supported capability**

---

## 4.13 `maker_id -> circle games / profile`

### Status
- **Implemented**

### 구현 요약
- `CircleClient::list_circle_games(maker_id)` - 특정 circle의 모든 게임 목록 반환
- `CircleClient::get_circle(maker_id, options)` - circle 제품 검색 (필터링 가능)
- `CircleClient::get_circle_profile(circle_id)` - circle 프로필 메타데이터 조회

### Endpoint list
1. `/maniax/api/=/circle/profile.json?circle_id={circle_id}` - circle 프로필
2. `/maniax/srf/=/free_word/.../work_category%5B0%5D/game/...` - circle 제품 검색

### 이 엔드포인트로 구현된 기능
- `maker_id -> all games by circle`
- `circle_id -> circle metadata (name, description, follower_count, etc.)`

### Keep / remove
- **Keep as supported capability**

---

## 5. 현재 문서에서 빠져 있었지만 저장소에는 구현된 항목

1. `title -> candidate RJ list`
2. `title -> best-match RJ metadata`
3. `product_id -> official titles (ko/ja/en/zh-cn/zh-tw)`
4. `product_id -> tags`
5. `product_id -> developer / publisher / release_year`
6. `product_id -> rating / review_count / star_counts`
7. `mixed hint normalization`
8. `browser fallback search`
9. `work classification / non-game filtering`
10. `circle_name -> maker_id`
11. `maker_id -> all games by circle`
12. `circle_id -> circle profile`
13. `product_id -> reviews`

---

## 6. 기존 문서와 표현이 달라져야 하는 항목

### 6.1 Thumbnail
기존 문서는 canonical img_main 경로 검증 중심이었지만,  
저장소 구현은 **JSON 미디어 필드 직접 파싱** 중심이다.

### 6.2 Screenshots
기존 문서는 HTML `Image:` 앵커 기반 설명이었지만,  
저장소 구현은 **JSON 배열 기반 스크린샷 수집**이 1차 경로다.

### 6.3 Tags
기존 문서에는 tags 항목이 없었다.  
저장소 구현은 **locale-aware merge + single stored array** 구조다.

---

## 7. 이 저장소에 없는 항목

현재 모든 주요 DLsite 기능이 구현되어 있다. 추가로 구현이 필요한 항목은 다음과 같다:

### 7.1 `circle_name -> maker_id` (다른 방식)
- 현재 `resolve_circle_name()`은 제품 검색을 통한 간접 해결 방식을 사용한다.
- DLsite에서 circle 검색 전용 API가 존재하는지 확인이 필요하다.

### 7.2 인증 기반 기능
- 로그인 필요 기능 (구매 내역, 위시리스트 등)은 현재 구현되지 않았다.

---

## 8. 추천 inventory (repo 기준)

### Core
1. `fetch_dlsite_by_rj(rj_code) -> StoreMetadata`
2. `fetch_dlsite_search_candidates(title) -> [RJ candidates]`
3. `resolve_store_search_candidates(title, store="dlsite") -> [MetadataCandidateResult]`
4. `resolve_store_title_candidate(title, store="dlsite") -> best candidate`
5. `resolve_circle_name(circle_name) -> maker_id`
6. `list_circle_games(maker_id) -> [RJ codes]`
7. `get_circle_profile(circle_id) -> CircleProfile`
8. `get_review(product_id) -> Reviews`

### Included fields in `StoreMetadata`
- titles: `official_title_ko`, `official_title_ja`, `official_title_en`, `original_title`
- ids: `dlsite_rj_code`
- maker/meta: `developer`, `publisher`, `release_year`
- media: `thumbnail_url`, `screenshots`
- classification helpers: `tags`
- quality signals: `rating_label`, `rating_value`, `review_count`, `rating_detail`
- navigation: `store_url`

### Circle-related fields
- `maker_id`: circle 식별자
- `circle_name`: circle 이름
- `circle_profile`: circle 메타데이터 (팔로워 수, 설명 등)

### Explicit non-core / not in repo
- 인증 기반 기능 (로그인 필요)
- 구매 내역 조회
- 위시리스트 관리

---

## 9. 구현 메모

### Suggested wording for tags
- `product_id -> tags`: **Implemented**
- Note: “Tags are collected from locale-specific `genres` payloads and merged into a single stored tag array. The repo does not currently persist per-language tag arrays separately.”

### Suggested wording for screenshots
- `product_id -> screenshots`: **Implemented**
- Note: “Screenshots are primarily collected from media arrays in product/info payloads, not from HTML-only anchor scraping.”

### Suggested wording for thumbnail
- `product_id -> thumbnail`: **Implemented**
- Note: “Thumbnail is resolved from product/info image fields and normalized into an absolute media URL.”

---