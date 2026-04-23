// aura-net/src/lib.rs

use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use adblock::Engine as AdblockEngine;
use once_cell::sync::Lazy;
use sha2::Digest;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

pub struct NetState {
    engine: AdblockEngine,
}

static NET_STATE: Lazy<Arc<RwLock<Option<NetState>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

pub enum InterceptDecision {
    Allow(Url),
    Block { reason: &'static str },
    Redirect(Url),
}

pub async fn init_adblock(urls: &[&str]) {
    let mut filter_set = FilterSet::new(false);
    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".aura")
        .join("lists");
    let _ = std::fs::create_dir_all(&cache_dir);

    for url in urls {
        let file_name = hex::encode(sha2::Sha256::digest(url.as_bytes()));
        let cache_path = cache_dir.join(file_name);

        let content = if let Ok(metadata) = std::fs::metadata(&cache_path) {
            if metadata.modified().map_or(false, |m| m.elapsed().map_or(false, |e| e.as_secs() < 86400)) {
                std::fs::read_to_string(&cache_path).ok()
            } else {
                None
            }
        } else {
            None
        };

        let content = match content {
            Some(c) => c,
            None => {
                if let Ok(resp) = reqwest::get(*url).await {
                    if let Ok(body) = resp.text().await {
                        let _ = std::fs::write(&cache_path, &body);
                        body
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
        };
        filter_set.add_filters(&content.lines().collect::<Vec<_>>(), ParseOptions::default());
    }

    let engine = AdblockEngine::from_filter_set(filter_set, true);
    let mut guard = NET_STATE.write().await;
    *guard = Some(NetState { engine });
}

pub async fn intercept(
    request_url: &Url,
    source_url: &Url,
    resource_type: &str,
) -> InterceptDecision {
    // Force HTTPS upgrade
    if request_url.scheme() == "http" {
        if let Ok(https) = request_url.to_string().replace("http://", "https://").parse::<Url>() {
            return InterceptDecision::Redirect(https);
        }
    }

    let guard = NET_STATE.read().await;
    if let Some(state) = guard.as_ref() {
        let request = Request::new(request_url.as_str(), source_url.as_str(), resource_type)
            .unwrap_or_else(|_| Request::new("", "", "").unwrap());

        let block_result = state.engine.check_network_request(&request);
        if block_result.matched {
            return InterceptDecision::Block { reason: "adblock_matched" };
        }
    }

    InterceptDecision::Allow(request_url.clone())
}
