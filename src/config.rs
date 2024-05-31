use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub urls: Vec<String>,
    pub output: String,
    pub method: String,
    pub timeout: u64,
    pub retries: usize,
    pub verbose: bool,
    pub force_http: bool,
    pub concurrency: usize,
}

impl Config {
    pub fn from_matches(matches: &ArgMatches) -> io::Result<Self> {
        let filename = matches.value_of("file").unwrap();
        let urls = Self::read_urls_from_file(filename)?;

        Ok(Self {
            urls,
            output: matches.value_of("output").unwrap().to_string(),
            method: matches.value_of("method").unwrap().to_string(),
            timeout: matches.value_of_t("timeout").unwrap_or(20),
            retries: matches.value_of_t("retries").unwrap_or(3),
            verbose: matches.is_present("verbose"),
            force_http: matches.is_present("force-http"),
            concurrency: matches.value_of_t("concurrency").unwrap_or(10),
        })
    }

    fn read_urls_from_file(filename: &str) -> io::Result<Vec<String>> {
        let path = Path::new(filename);
        let mut urls = Vec::new();

        if filename.ends_with(".json") {
            let file = File::open(path)?;
            let url_list: UrlList = serde_json::from_reader(file)?;
            urls = url_list.urls;
        } else {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                urls.push(line?);
            }
        }

        Ok(urls)
    }

    pub fn normalize_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            if self.force_http {
                return url.replacen("https://", "http://", 1);
            }
            return url.to_string();
        } else {
            if self.force_http {
                return format!("http://{}", url);
            }
            return format!("https://{}", url);
        }
    }
}

#[derive(Deserialize)]
struct UrlList {
    urls: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct CacheInfo {
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub method: String,
}
