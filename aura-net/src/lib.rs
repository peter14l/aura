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
    Redirect(Url), // For HTTPS upgrades
}

pub async fn intercept(
    request_url: &Url,
    source_url: &Url,
    resource_type: &str,
) -> InterceptDecision {
    let block_result =
        ADBLOCK.check_network_urls(request_url.as_str(), source_url.as_str(), resource_type);

    if block_result.matched {
        return InterceptDecision::Block {
            reason: "adblock_matched",
        };
    }

    // Force HTTPS upgrade
    if request_url.scheme() == "http" {
        if let Ok(https) = request_url
            .clone()
            .into_string()
            .replace("http://", "https://")
            .parse::<Url>()
        {
            return InterceptDecision::Redirect(https);
        }
    }

    InterceptDecision::Allow(request_url.clone())
}

fn load_or_fetch_lists(urls: &[&str]) -> Vec<String> {
    let mut all_rules = Vec::new();
    let cache_dir = dirs::home_dir().unwrap().join(".aura").join("lists");
    std::fs::create_dir_all(&cache_dir).unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    for url in urls {
        let file_name = hex::encode(sha2::Sha256::digest(url.as_bytes()));
        let cache_path = cache_dir.join(file_name);

        let mut use_cache = false;
        if let Ok(metadata) = std::fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if modified.elapsed().unwrap().as_secs() < 86400 {
                    use_cache = true;
                }
            }
        }

        let content = if use_cache {
            std::fs::read_to_string(&cache_path).unwrap()
        } else {
            let body = rt.block_on(async {
                reqwest::get(*url).await.unwrap().text().await.unwrap()
            });
            std::fs::write(&cache_path, &body).unwrap();
            body
        };

        all_rules.extend(content.lines().map(|s| s.to_string()));
    }

    all_rules
}
