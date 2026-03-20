use crate::interface::query::Order;

use super::super::search::macros::*;

#[derive(Default)]
pub struct CircleQuery {
    /// Display lang
    pub order: Option<Order>,
    pub options: Option<Vec<String>>,
    /// 30, 50 or 100
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

impl CircleQuery {
    pub(super) fn to_path(&self, circle_id: &str) -> String {
        let mut path = "/circle/profile/=".to_string();

        push_option_array!(path, self, options);
        push_option!(path, self, per_page);
        // TODO: Investigate this
        path.push_str("/show_type/3/hd/1/without_order/1");
        push_option!(path, self, page);
        path.push_str("/maker_id/");
        path.push_str(circle_id);
        path.push_str(".html");
        push_option!(path, self, order);

        path
    }
}
#[cfg(test)]
mod tests {
    use crate::{client::circle::CircleQuery, interface::query::Order};

    #[test]
    fn circle_param() {
        assert_eq!(
            "/circle/profile/=/show_type/3/hd/1/without_order/1/maker_id/RG24350.html",
            CircleQuery::default().to_path("RG24350")
        );
    }
    #[test]
    fn circle_param_1() {
        assert_eq!(
            "/circle/profile/=/show_type/3/hd/1/without_order/1/page/2/maker_id/RG24350.html",
            CircleQuery {
                page: Some(2),
                ..Default::default()
            }
            .to_path("RG24350")
        );
    }
    #[test]
    fn circle_param_2() {
        assert_eq!(
            "/circle/profile/=/show_type/3/hd/1/without_order/1/page/2/maker_id/RG24350.html/order/price",
            CircleQuery {
                page: Some(2),
                order: Some(Order::Price),
                ..Default::default()
            }
            .to_path("RG24350")
        );
    }

    #[test]
    fn per_page_appears_exactly_once() {
        let path = CircleQuery {
            per_page: Some(30),
            ..Default::default()
        }
        .to_path("RG24350");
        let count = path.matches("/per_page/").count();
        assert_eq!(1, count, "per_page should appear exactly once, got: {path}");
    }
}
