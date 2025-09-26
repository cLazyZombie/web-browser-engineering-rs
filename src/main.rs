//! Web Browser Engineering in Rust

use std::env;
use anyhow::Result;
use web_browser_engineering_rs::url::Url;

fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if URL argument is provided
    if args.len() != 2 {
        eprintln!("Usage: {} <URL>", args[0]);
        eprintln!("Example: {} http://example.org/", args[0]);
        std::process::exit(1);
    }

    // Parse and request the URL
    let url = Url::new(&args[1])?;
    let response_body = url.request()?;

    // Print the response body
    println!("{}", response_body.as_str());

    Ok(())
}
