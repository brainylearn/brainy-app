use scraper::{Html, Selector};

pub struct HighlightContent {
    pub id: String,
    pub inner_html: String,
}

pub fn parse_highlights(html: &str) -> Vec<HighlightContent> {
    let document = Html::parse_fragment(html);
    let selector = Selector::parse("highlight").expect("Invalid selector");
    document
        .select(&selector)
        .filter_map(|el| {
            let id = el.attr("highlight-id")?.to_string();
            Some(HighlightContent {
                id,
                inner_html: el.inner_html(),
            })
        })
        .collect()
}
