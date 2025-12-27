use regex::Regex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use serde::Serialize;
use chrono::Utc;
use futures::StreamExt;

#[derive(Debug, Serialize, Clone)]
pub struct Intel {
    pub source: String,
    pub title: String,
    pub link: String,
    pub discovered_at: String,
    pub category: String,
    pub status: String,
    pub raw_onion: String,
}

const LINK_CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const SECTOR_PAGE_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_RETRIES: usize = 2;

pub async fn attempt_check_with_timeout(
    client: &reqwest::Client,
    link: &str,
    keyword_regex: &Regex,
) -> (Option<String>, String) {
    match tokio::time::timeout(LINK_CHECK_TIMEOUT, client.head(link).send()).await {
        Ok(Ok(head_resp)) => {
            let status_code = head_resp.status().as_u16();
            let mut cat = "Unknown".to_string();
            if status_code == 200 {
                if let Ok(get_result) = tokio::time::timeout(
                    LINK_CHECK_TIMEOUT,
                    client.get(link).send(),
                ).await {
                    if let Ok(get_resp) = get_result {
                        if let Ok(body) = get_resp.text().await {
                            if let Some(mat) = keyword_regex.find(&body) {
                                cat = mat.as_str().to_string();
                            }
                        }
                    }
                }
                (Some("Online".to_string()), cat)
            } else {
                (Some(format!("Offline({})", status_code)), cat)
            }
        }
        Ok(Err(e)) => {
            log::warn!("HEAD request failed for {}: {}", link, e);
            (None, "Unknown".to_string())
        }
        Err(_) => {
            log::warn!("HEAD request timeout for {}", link);
            (None, "Unknown".to_string())
        }
    }
}

pub async fn scan_sector_improved(
    client: &reqwest::Client,
    url: &str,
    source_name: &str,
    sector_semaphore: Arc<Semaphore>,
    link_parallelism: usize,
) -> Vec<Intel> {
    let _permit = match sector_semaphore.acquire_owned().await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to acquire semaphore for {}: {}", source_name, e);
            return Vec::new();
        }
    };

    log::info!("[*] Probing Sector: {}...", source_name);

    let resp_text = match tokio::time::timeout(SECTOR_PAGE_TIMEOUT, client.get(url).send()).await {
        Ok(Ok(r)) => match r.text().await {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to read body for {}: {}", source_name, e);
                return Vec::new();
            }
        },
        Ok(Err(e)) => {
            log::error!("Sector {} unreachable: {}", source_name, e);
            return Vec::new();
        }
        Err(_) => {
            log::error!("Sector {} timeout", source_name);
            return Vec::new();
        }
    };

    let onion_regex = Regex::new(r"([a-z2-7]{56}\.onion)").unwrap();
    let keyword_regex = Regex::new(r"(?i)\b(market|vendor|leak|forum|wallet|escrow|hosting|payment|shop|card)\b").unwrap();

    let mut discovered = std::collections::HashSet::new();
    for mat in onion_regex.find_iter(&resp_text) {
        discovered.insert(mat.as_str().to_string());
    }

    let document = scraper::Html::parse_document(&resp_text);
    let selector = scraper::Selector::parse("a").unwrap();
    let mut candidates: Vec<(String, String)> = Vec::new();

    for element in document.select(&selector) {
        let title = element.text().collect::<Vec<_>>().join("").trim().to_string();
        if let Some(href) = element.value().attr("href") {
            if href.contains(".onion") {
                let raw_onion = onion_regex.captures(href)
                    .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                    .unwrap_or_else(|| href.to_string());
                candidates.push((raw_onion.clone(), title));
            }
        }
    }

    for onion in discovered {
        if !candidates.iter().any(|(o, _)| o == &onion) {
            candidates.push((onion, String::new()));
        }
    }

    let link_semaphore = Arc::new(Semaphore::new(link_parallelism));
    let results: Vec<Intel> = futures::stream::iter(candidates.into_iter())
        .map(|(onion, title)| {
            let client = client.clone();
            let keyword_regex = keyword_regex.clone();
            let source_name = source_name.to_string();
            let link_semaphore = link_semaphore.clone();

            async move {
                let _link_permit = link_semaphore.acquire_owned().await.ok()?;
                let link = format!("http://{}", onion);
                let mut status = None;
                let mut category = "Unknown".to_string();
                let mut attempts = 0;

                while attempts < MAX_RETRIES && status.is_none() {
                    let (s, c) = attempt_check_with_timeout(&client, &link, &keyword_regex).await;
                    if s.is_some() {
                        status = s;
                        category = c;
                        break;
                    }
                    attempts += 1;
                    if attempts < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(500 * (2_u64.pow(attempts as u32 - 1)))).await;
                    }
                }

                let final_status = status.unwrap_or_else(|| "Unknown".to_string());
                let final_title = if title.is_empty() { "Raw Onion Discovery".to_string() } else { title };

                Some(Intel {
                    source: format!("{} - {}", source_name, category.clone()),
                    title: final_title,
                    link: normalize_url(&link),
                    discovered_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    category,
                    status: final_status,
                    raw_onion: onion,
                })
            }
        })
        .buffer_unordered(link_parallelism * 2)
        .filter_map(|r| async move { r })
        .collect()
        .await;

    log::info!("[+] Sector {} done ({} items)", source_name, results.len());
    results
}

fn normalize_url(url: &str) -> String {
    let mut s = url.to_string();
    for prefix in &["http://", "https://", "//"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }
    s.trim_end_matches('/').to_string()
}
