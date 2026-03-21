//! Site/host abstraction for DLsite subdomains.

/// Represents a DLsite site/subdomain.
///
/// Each variant corresponds to a content segment on `https://www.dlsite.com/`.
/// Use [`Site::base_url()`] to get the full base URL for that site.
#[derive(Clone, Debug, PartialEq)]
pub enum Site {
    /// General/home site (`/home`)
    Home,
    /// Adult doujin site (`/maniax`) — the default
    Maniax,
    /// Books site (`/books`)
    Books,
    /// Software/utility site (`/soft`)
    Soft,
    /// Professional/commercial site (`/pro`)
    Pro,
    /// Android app site (`/appx`)
    Appx,
    /// Comics/manga site (`/comic`)
    Comic,
    /// Custom segment — use when the above variants do not cover your target.
    Custom(String),
}

impl Default for Site {
    fn default() -> Self {
        Site::Maniax
    }
}

impl Site {
    /// Returns the full base URL for this site.
    ///
    /// # Examples
    /// ```
    /// use dlsite_gamebox::interface::site::Site;
    /// assert_eq!(Site::Maniax.base_url(), "https://www.dlsite.com/maniax");
    /// assert_eq!(Site::Books.base_url(), "https://www.dlsite.com/books");
    /// ```
    pub fn base_url(&self) -> String {
        let segment = match self {
            Site::Home => "home",
            Site::Maniax => "maniax",
            Site::Books => "books",
            Site::Soft => "soft",
            Site::Pro => "pro",
            Site::Appx => "appx",
            Site::Comic => "comic",
            Site::Custom(s) => s.as_str(),
        };
        format!("https://www.dlsite.com/{}", segment)
    }
}

#[cfg(test)]
mod tests {
    use super::Site;

    #[test]
    fn maniax_base_url() {
        assert_eq!("https://www.dlsite.com/maniax", Site::Maniax.base_url());
    }

    #[test]
    fn home_base_url() {
        assert_eq!("https://www.dlsite.com/home", Site::Home.base_url());
    }

    #[test]
    fn books_base_url() {
        assert_eq!("https://www.dlsite.com/books", Site::Books.base_url());
    }

    #[test]
    fn soft_base_url() {
        assert_eq!("https://www.dlsite.com/soft", Site::Soft.base_url());
    }

    #[test]
    fn pro_base_url() {
        assert_eq!("https://www.dlsite.com/pro", Site::Pro.base_url());
    }

    #[test]
    fn appx_base_url() {
        assert_eq!("https://www.dlsite.com/appx", Site::Appx.base_url());
    }

    #[test]
    fn comic_base_url() {
        assert_eq!("https://www.dlsite.com/comic", Site::Comic.base_url());
    }

    #[test]
    fn custom_base_url() {
        assert_eq!(
            "https://www.dlsite.com/girls",
            Site::Custom("girls".to_string()).base_url()
        );
    }

    #[test]
    fn default_is_maniax() {
        assert_eq!(Site::Maniax, Site::default());
    }
}
