use reqwest::Client;
use tokio::time::{self, Duration};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use trust_dns_resolver::TokioAsyncResolver;
use crate::config::CacheInfo;

pub async fn check_cache_headers(
    url: &str, client: &Client, method: &str, timeout: u64, retries: usize, verbose: bool,
    resolver: &TokioAsyncResolver, headers: &[String]
) -> Option<CacheInfo> {
    if verbose {
        println!("Requesting URL: {}", url);
    }

    let mut rng = StdRng::from_entropy();
    let mut backoff = Duration::from_millis(500);

    for attempt in 0..=retries {
        let request = match method {
            "GET" => client.get(url),
            "POST" => client.post(url),
            _ => client.get(url),
        };

        // Resolve DNS before making the request
        let domain = url.split('/').nth(2).unwrap_or("");
        if let Err(e) = resolver.lookup_ip(domain).await {
            if verbose {
                eprintln!("DNS error for {}: {}", url, e);
            }
            if attempt < retries {
                time::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
                continue;
            } else {
                return None;
            }
        }

        let _response = match time::timeout(Duration::from_secs(timeout), request.send()).await {
            Ok(Ok(response)) => {
                if verbose {
                    println!("Received response for URL: {}", url);
                }

                let mut cache_info = std::collections::HashMap::new();

                for header in headers {
                    if let Some(value) = response.headers().get(header) {
                        cache_info.insert(header.to_string(), value.to_str().unwrap().to_string());
                        if verbose {
                            println!("Found header {}: {}", header, value.to_str().unwrap());
                        }
                    }
                }

                if let Ok(body) = response.text().await {
                    if body.contains("served from cache") || body.contains("X-Cache: HIT") {
                        cache_info.insert("Body-Cache-Indicator".to_string(), "Detected".to_string());
                        if verbose {
                            println!("Cache indicator found in body for URL: {}", url);
                        }
                    }
                }

                if cache_info.is_empty() {
                    if verbose {
                        println!("No caching headers found for URL: {}", url);
                    }
                    return None;
                } else {
                    return Some(CacheInfo {
                        url: url.to_string(),
                        headers: cache_info,
                        method: method.to_string(),
                    });
                }
            }
            Ok(Err(e)) => {
                if verbose {
                    eprintln!("Error requesting {}: {}", url, e);
                }
                if attempt < retries {
                    if verbose {
                        println!("Retrying... (attempt {}/{})", attempt + 1, retries);
                    }
                } else {
                    return None;
                }
            }
            Err(e) => {
                if verbose {
                    eprintln!("Timeout requesting {}: {}", url, e);
                }
                if attempt < retries {
                    if verbose {
                        println!("Retrying... (attempt {}/{})", attempt + 1, retries);
                    }
                } else {
                    return None;
                }
            }
        };

        // Randomized delay before next attempt
        let delay = rng.gen_range(1000..5000);
        if verbose {
            println!("Waiting for {} milliseconds before next attempt", delay);
        }
        time::sleep(Duration::from_millis(delay)).await;
    }

    None
}
