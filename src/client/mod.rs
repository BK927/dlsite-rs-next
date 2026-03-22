use crate::cache::ResponseCache;
use crate::error::{DlsiteError, Result};
use crate::interface::query::Language;
use crate::interface::site::Site;
use crate::retry::RetryConfig;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub mod product;
pub mod product_api;
pub mod ranking;

#[cfg(feature = "search-html")]
pub mod circle;
#[cfg(feature = "search-html")]
pub mod search;

#[cfg(feature = "cookie-store")]
pub mod auth;
#[cfg(feature = "cookie-store")]
pub mod play;
#[cfg(feature = "cookie-store")]
pub mod user;

/// Rate limiter state for controlling request frequency.
#[derive(Debug)]
struct RateLimiter {
    last_request: Instant,
    min_interval: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified minimum interval between requests.
    fn new(min_interval: Duration) -> Self {
        Self {
            // Initialize with elapsed time to allow first request immediately
            last_request: Instant::now() - min_interval,
            min_interval,
        }
    }

    /// Wait until rate limit allows, then record the request time.
    async fn throttle(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        self.last_request = Instant::now();
    }
}

/// API client for DLsite.
#[derive(Clone, Debug)]
pub struct DlsiteClient {
    client: reqwest::Client,
    base_url: String,
    /// Rate limiter to prevent IP bans (2 requests per second by default)
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// Response cache for caching HTTP responses
    cache: ResponseCache,
    /// Retry configuration for automatic retries
    retry_config: RetryConfig,
    /// Default locale used for locale-aware API calls
    default_locale: Language,
}

impl Default for DlsiteClient {
    fn default() -> Self {
        Self::new("https://www.dlsite.com/maniax")
    }
}

/// Builder for DlsiteClient with customizable configuration
pub struct DlsiteClientBuilder {
    base_url: String,
    pool_max_idle_per_host: usize,
    timeout: Duration,
    cache_capacity: usize,
    cache_ttl: Duration,
    retry_config: RetryConfig,
    default_locale: Language,
}

impl DlsiteClientBuilder {
    /// Create a new builder with default settings
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            pool_max_idle_per_host: 10,
            timeout: Duration::from_secs(30),
            cache_capacity: 100,
            cache_ttl: Duration::from_secs(3600),
            retry_config: RetryConfig::default(),
            default_locale: Language::Jp,
        }
    }

    /// Set the default locale for locale-aware API calls
    pub fn locale(mut self, locale: Language) -> Self {
        self.default_locale = locale;
        self
    }

    /// Set the maximum number of idle connections per host
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the cache capacity and TTL
    pub fn cache(mut self, capacity: usize, ttl: Duration) -> Self {
        self.cache_capacity = capacity;
        self.cache_ttl = ttl;
        self
    }

    /// Set the retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set the target DLsite site. Overrides the base URL set in [`Self::new`].
    pub fn site(mut self, site: Site) -> Self {
        self.base_url = site.base_url();
        self
    }

    /// Build the DlsiteClient.
    ///
    /// Returns an error if the HTTP client cannot be constructed
    /// (e.g., invalid TLS configuration).
    pub fn build(self) -> Result<DlsiteClient> {
        let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        #[allow(unused_mut)]
        let mut builder = reqwest::Client::builder()
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .timeout(self.timeout)
            .user_agent(&user_agent);

        #[cfg(feature = "cookie-store")]
        {
            builder = builder.cookie_store(true);
        }

        let client = builder
            .build()
            .map_err(|e| DlsiteError::Parse(format!("Failed to build HTTP client: {}", e)))?;

        Ok(DlsiteClient {
            client,
            base_url: self.base_url,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(Duration::from_millis(500)))),
            cache: ResponseCache::new(self.cache_capacity, self.cache_ttl),
            retry_config: self.retry_config,
            default_locale: self.default_locale,
        })
    }
}

impl DlsiteClient {
    /// Create a new DLsite client with a custom base URL.
    ///
    /// Typical base URL is `https://www.dlsite.com/maniax` and you should be able to access any
    /// products using this URL, so usually you don't use this method and just use the default.
    ///
    /// The client is configured with:
    /// - Connection pool: 10 idle connections per host
    /// - Timeout: 30 seconds
    /// - User-Agent: dlsite-rs/\<version\>
    /// - Rate limit: 2 requests per second
    /// - Cache: 100 entries with 1 hour TTL
    /// - Retry: 3 attempts with exponential backoff
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be constructed. For fallible construction,
    /// use [`DlsiteClient::try_new`].
    pub fn new(base_url: &str) -> Self {
        DlsiteClientBuilder::new(base_url)
            .build()
            .expect("Failed to build DlsiteClient")
    }

    /// Create a new DLsite client, returning an error on failure.
    ///
    /// This is the fallible version of [`DlsiteClient::new`].
    pub fn try_new(base_url: &str) -> Result<Self> {
        DlsiteClientBuilder::new(base_url).build()
    }

    /// Create a new DLsite client targeting a specific [`Site`].
    ///
    /// Equivalent to `DlsiteClient::new(&site.base_url())`.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be constructed.
    pub fn for_site(site: Site) -> Self {
        DlsiteClientBuilder::new(&site.base_url())
            .build()
            .expect("Failed to build DlsiteClient")
    }

    /// Create a builder for customizing the client configuration
    pub fn builder(base_url: &str) -> DlsiteClientBuilder {
        DlsiteClientBuilder::new(base_url)
    }

    /// Convenient method to make a http GET request using the client.
    ///
    /// This method respects the rate limiter to prevent IP bans, uses caching, and retries on failure.
    /// Rate limit: 2 requests per second (500ms between requests)
    /// Cache: 100 entries with 1 hour TTL
    /// Retry: 3 attempts with exponential backoff for retryable errors
    pub async fn get(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);

        // Check cache first
        if let Some(cached) = self.cache.get(&url) {
            return Ok(cached);
        }

        // Retry loop
        let mut last_error = None;
        for attempt in 0..=self.retry_config.max_retries {
            // Rate limiting (thread-safe)
            self.rate_limiter.lock().await.throttle().await;

            match self.client.get(&url).send().await {
                Ok(response) => {
                    // Check HTTP status code
                    let status = response.status();
                    if status == 429 {
                        let err = DlsiteError::RateLimit(
                            "Too many requests, please retry later".to_string(),
                        );
                        if attempt < self.retry_config.max_retries
                            && self.retry_config.is_retryable(&err)
                        {
                            last_error = Some(err);
                            let delay = self.retry_config.calculate_delay(attempt);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(err);
                    }
                    if status == 401 {
                        return Err(DlsiteError::AuthRequired(
                            "HTTP 401 Unauthorized".to_string(),
                        ));
                    }
                    if status == 403 {
                        return Err(DlsiteError::AuthRequired("HTTP 403 Forbidden".to_string()));
                    }
                    if !status.is_success() {
                        let err = DlsiteError::HttpStatus(status.as_u16());
                        if attempt < self.retry_config.max_retries
                            && self.retry_config.is_retryable(&err)
                        {
                            last_error = Some(err);
                            let delay = self.retry_config.calculate_delay(attempt);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(err);
                    }

                    let body = response.text().await?;

                    // Cache the response
                    self.cache.insert(url, body.clone());

                    return Ok(body);
                }
                Err(e) => {
                    let err = DlsiteError::from(e);
                    if attempt < self.retry_config.max_retries
                        && self.retry_config.is_retryable(&err)
                    {
                        last_error = Some(err);
                        let delay = self.retry_config.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        // If we exhausted all retries, return the last error
        Err(last_error.unwrap_or_else(|| DlsiteError::Parse("Unknown error".to_string())))
    }

    /// Similar to `get`, but this method does not prepend the base URL.
    pub async fn get_raw(&self, url: &str) -> Result<String> {
        let body = self.client.get(url).send().await?.text().await?;
        Ok(body)
    }

    /// Clear the response cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get the number of entries in the cache
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Set the retry configuration
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }

    /// Get the current retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Get the default locale configured on this client
    pub fn default_locale(&self) -> &Language {
        &self.default_locale
    }
}

/// These methods return a "sub-client".
/// The sub-client has a DlsiteClient reference inside and has implementations of fetch and parse focused on certain purposes.
impl DlsiteClient {
    /// Get a client to fetch product info using 'scraping' method. For more information, see [`product::ProductClient`].
    pub fn product(&self) -> product::ProductClient<'_> {
        product::ProductClient { c: self }
    }

    /// Get a client to fetch product info using 'api' method. For more information, see
    /// [`product_api::ProductApiClient`].
    pub fn product_api(&self) -> product_api::ProductApiClient<'_> {
        product_api::ProductApiClient { c: self }
    }

    /// Get a client to fetch circle info. For more information, see [`circle::CircleClient`].
    ///
    /// **Requires `search-html` feature flag.**
    #[cfg(feature = "search-html")]
    pub fn circle(&self) -> circle::CircleClient<'_> {
        circle::CircleClient { c: self }
    }

    /// Get a client to search things. For more information, see [`search::SearchClient`].
    ///
    /// **Requires `search-html` feature flag.**
    #[cfg(feature = "search-html")]
    pub fn search(&self) -> search::SearchClient<'_> {
        search::SearchClient::new(self)
    }

    /// Get a client for ranking data. For more information, see [`ranking::RankingClient`].
    ///
    /// Note: ranking endpoints are not yet implemented — see `docs/dlsite_gap_analysis.md`.
    pub fn ranking(&self) -> ranking::RankingClient<'_> {
        ranking::RankingClient { c: self }
    }

    /// Get a client for authentication. Requires the `cookie-store` feature.
    ///
    /// Note: auth endpoints are not yet implemented — see `docs/dlsite_gap_analysis.md`.
    #[cfg(feature = "cookie-store")]
    pub fn auth(&self) -> auth::AuthClient<'_> {
        auth::AuthClient { c: self }
    }

    /// Get a client for DLsite Play streaming. Requires the `cookie-store` feature.
    ///
    /// Note: Play endpoints are not yet implemented — see `docs/dlsite_gap_analysis.md`.
    #[cfg(feature = "cookie-store")]
    pub fn play(&self) -> play::PlayClient<'_> {
        play::PlayClient { c: self }
    }

    /// Get a client for user library and purchase data. Requires the `cookie-store` feature.
    ///
    /// Note: user endpoints are not yet implemented — see `docs/dlsite_gap_analysis.md`.
    #[cfg(feature = "cookie-store")]
    pub fn user(&self) -> user::UserClient<'_> {
        user::UserClient { c: self }
    }
}
