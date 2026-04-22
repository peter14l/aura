// aura-net/src/lib.rs

use adblock::engine::Engine as AdblockEngine;
use adblock::lists::{FilterSet, ParseOptions};
use once_cell::sync::Lazy;
use url::Url;

static ADBLOCK: Lazy<AdblockEngine> = Lazy::new(|| {
    let raw_rules = load_or_fetch_lists(&[
        "https://easylist.to/easylist/easylist.txt",
        "https://easylist.to/easylist/easyprivacy.txt",
        "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt",
    ]);

    let mut filter_set = FilterSet::new(false);
    filter_set.add_filters(&raw_rules, ParseOptions::default());
    AdblockEngine::from_filter_set(filter_set, true)
});

pub enum InterceptDecision {
    Allow(Url),
    Block { reason: &'static str },
    Redirect(Url),  // For HTTPS upgrades
}

pub async fn intercept(
    request_url: &Url,
    source_url: &Url,
    resource_type: &str,
) -> InterceptDecision {
    let block_result = ADBLOCK.check_network_urls(
        request_url.as_str(),
        source_url.as_str(),
        resource_type,
    );

    if block_result.matched {
        return InterceptDecision::Block {
            reason: "adblock_matched",
        };
    }

    // Force HTTPS upgrade
    if request_url.scheme() == "http" {
        if let Ok(https) = request_url.clone().into_string().replace("http://", "https://").parse::<Url>() {
            return InterceptDecision::Redirect(https);
        }
    }

    InterceptDecision::Allow(request_url.clone())
}

fn load_or_fetch_lists(_urls: &[&str]) -> Vec<String> {
    // TODO: Implement list fetching with 24h TTL cache
    // Returns concatenated filter rules as lines
    vec![]
}
