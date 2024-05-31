mod config;
mod dns_resolver;
mod cache_checker;

use config::Config;
use dns_resolver::create_resolver;
use cache_checker::check_cache_headers;
use clap::Command;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, Semaphore};
use tokio::task;
use tokio::time::Duration;
use std::fs::File;
use std::io::{self, BufRead, Result as IoResult, Write};
use tokio::signal;

#[tokio::main]
async fn main() -> IoResult<()> {
    let matches = Command::new("cache_scraper")
        .version("0.1.0")
        .about("Scrapes URLs for cache headers")
        .arg(clap::Arg::new("file").help("The file containing URLs to scrape").required(true).index(1))
        .arg(clap::Arg::new("output").short('o').long("output").help("The file to save results to").takes_value(true).required(true))
        .arg(clap::Arg::new("method").short('m').long("method").help("HTTP method to use (GET or POST)").takes_value(true).default_value("GET"))
        .arg(clap::Arg::new("timeout").short('t').long("timeout").help("Request timeout in seconds").takes_value(true).default_value("20"))
        .arg(clap::Arg::new("retries").short('r').long("retries").help("Number of retries for failed requests").takes_value(true).default_value("3"))
        .arg(clap::Arg::new("verbose").short('v').long("verbose").help("Enable verbose output"))
        .arg(clap::Arg::new("force-http").long("force-http").help("Force HTTP instead of HTTPS"))
        .arg(clap::Arg::new("concurrency").long("concurrency").help("Maximum number of concurrent requests").takes_value(true).default_value("10"))
        .arg(clap::Arg::new("headers").short('H').long("headers").help("Comma-separated list of headers to check or path to file containing headers").takes_value(true))
        .get_matches();

    let config = Config::from_matches(&matches)?;

    let headers = read_headers(matches.value_of("headers"))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .build()
        .expect("Failed to build client");

    let resolver = create_resolver().expect("Failed to create DNS resolver");

    let pb = ProgressBar::new(config.urls.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .progress_chars("#>-")
    );

    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let (tx, mut rx) = mpsc::channel(100);
    let mut tasks = vec![];

    // Open the output file in write mode initially to clear it, then in append mode for subsequent writes
    {
        let mut file = File::create(&config.output)?;
        writeln!(file, "[").expect("Failed to write opening bracket");
    }
    let output_file = OpenOptions::new().append(true).open(&config.output)?;
    let output_file = Arc::new(Mutex::new(output_file));

    let first_result = Arc::new(Mutex::new(true)); // To track the first result

    // Spawn a task to handle Ctrl+C
    let output_file_clone = Arc::clone(&output_file);
    let handle = tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        append_closing_bracket(&output_file_clone);
        std::process::exit(0);
    });

    for url in &config.urls {
        let normalized_url = config.normalize_url(url);

        let client = client.clone();
        let pb = pb.clone();
        let semaphore = Arc::clone(&semaphore);
        let resolver = resolver.clone();
        let config = config.clone();
        let tx = tx.clone();
        let output_file = Arc::clone(&output_file); // Clone the Arc, not the file handle
        let headers = headers.clone();
        let first_result = Arc::clone(&first_result);

        tasks.push(task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = check_cache_headers(&normalized_url, &client, &config.method, config.timeout, config.retries, config.verbose, &resolver, &headers).await;
            if let Some(cache_info) = &result {
                // Write the result to the output file in a thread-safe manner
                let mut file = output_file.lock().unwrap();
                let mut is_first = first_result.lock().unwrap();
                let json = serde_json::to_string(&cache_info).unwrap();
                if *is_first {
                    if let Err(e) = write!(file, "{}", json) {
                        eprintln!("Failed to write to output file: {}", e);
                    }
                    *is_first = false;
                } else {
                    if let Err(e) = write!(file, ",{}", json) {
                        eprintln!("Failed to write to output file: {}", e);
                    }
                }
            }
            tx.send(result).await.unwrap();
            pb.inc(1);
        }));
    }

    drop(tx);

    let mut results = vec![];

    while let Some(result) = rx.recv().await {
        if let Some(cache_info) = result {
            pb.println(format!("Positive result for URL: {}", cache_info.url));
            for (header, value) in &cache_info.headers {
                pb.println(format!("  {}: {}", header, value));
            }
            results.push(cache_info);
        }
    }

    pb.finish_with_message("Done");

    append_closing_bracket(&output_file);

    if config.verbose {
        println!("Results saved to {}", config.output);
    }

    handle.await.expect("Ctrl+C handler failed");

    Ok(())
}

fn read_headers(header_arg: Option<&str>) -> IoResult<Vec<String>> {
    let default_headers = vec![
        "Cache-Control".to_string(), "Expires".to_string(), "ETag".to_string(),
        "Last-Modified".to_string(), "Age".to_string(), "Pragma".to_string(),
        "Vary".to_string(), "Server-Timing".to_string(), "CF-Cache-Status".to_string(),
        "CF-Ray".to_string(), "X-Cache".to_string(), "X-Cache-Lookup".to_string(),
        "X-Varnish".to_string(), "X-Cache-Remote".to_string(),
    ];

    if let Some(header_arg) = header_arg {
        if std::path::Path::new(header_arg).exists() {
            let file = File::open(header_arg)?;
            let reader = io::BufReader::new(file);
            let headers = reader.lines().collect::<Result<Vec<_>, _>>()?;
            Ok(headers)
        } else {
            Ok(header_arg.split(',').map(String::from).collect())
        }
    } else {
        Ok(default_headers)
    }
}

fn append_closing_bracket(output_file: &Arc<Mutex<File>>) {
    let mut file = output_file.lock().unwrap();
    if let Err(e) = writeln!(file, "]") {
        eprintln!("Failed to write closing bracket: {}", e);
    }
}
