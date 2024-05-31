# Cerberus Cache Scraper

Cerberus Cache Scraper is a Rust-based tool designed to vigilantly scrape URLs for specific cache headers. Named after the mythical multi-headed dog, Cerberus, it handles multiple URLs concurrently and supports customizable headers. The tool writes the results to a specified output file in JSON format and gracefully terminates with `Ctrl+C` by appending the closing bracket to the JSON output.

## Features

- Scrapes URLs for cache-related headers.
- Supports concurrent requests.
- Configurable request method, timeout, and retries.
- Handles `Ctrl+C` to ensure the output file is properly formatted.
- Supports both inline header lists and header files.

## Installation

To build and run Cerberus Cache Scraper, you need to have Rust and Cargo installed. You can install Rust and Cargo from [here](https://www.rust-lang.org/tools/install).

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/your-repo/cerberus_cache_scraper.git
cd cerberus_cache_scraper
```

Build the project:

```bash
cargo build --release
```

## Usage

The following is the usage information for Cerberus Cache Scraper:

```bash
USAGE:
    cerberus_cache_scraper [OPTIONS] --output <output> <file>

ARGS:
    <file>    The file containing URLs to scrape

OPTIONS:
        --concurrency <concurrency>    Maximum number of concurrent requests [default: 10]
        --force-http                   Force HTTP instead of HTTPS
    -h, --help                         Print help information
    -H, --headers <headers>            Comma-separated list of headers to check or path to file
                                       containing headers
    -m, --method <method>              HTTP method to use (GET or POST) [default: GET]
    -o, --output <output>              The file to save results to
    -r, --retries <retries>            Number of retries for failed requests [default: 3]
    -t, --timeout <timeout>            Request timeout in seconds [default: 20]
    -v, --verbose                      Enable verbose output
    -V, --version                      Print version information

```

## Examples

Basic Usage:

To scrape URLs listed in urls.txt and save the results to results.json:

```bash
cargo run -- urls.txt -o results.json
```

Custom Headers:

To specify custom headers directly:

```bash
cargo run -- urls.txt -o results.json -H "Cache-Control,Expires,ETag"
```

To specify custom headers from a file headers.txt:

```bash
cargo run -- urls.txt -o results.json -H headers.txt
```

Using Different HTTP Method:

To use the POST method instead of GET:

```bash
cargo run -- urls.txt -o results.json -m POST
```

Verbose Mode:

To enable verbose output:

```bash
cargo run -- urls.txt -o results.json -v
```

Force HTTP:

To force HTTP instead of HTTPS:

```bash
cargo run -- urls.txt -o results.json --force-http
```

Concurrency:

To set the number of concurrent requests:

```bash
cargo run -- urls.txt -o results.json --concurrency 20
```

## Contributing

If you have suggestions for improvements or encounter any issues, please feel free to open an issue or submit a pull request. Even though i have no intrest in updating this or understand how pull requests work lol.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
