// Example demonstrating how the URL request works
use web_browser_engineering_rs::url::Url;

fn main() -> anyhow::Result<()> {
    // Parse a URL
    let url = Url::new("http://example.com/path")?;

    // Make a request (in tests, this returns a fake connection)
    let mut conn = url.request()?;

    // The connection has already sent the HTTP request:
    // GET /path HTTP/1.0\r\n
    // Host: example.com\r\n
    // \r\n

    // In tests, we can verify what was sent
    #[cfg(test)]
    {
        let written = conn.get_written_data().unwrap();
        let request = String::from_utf8(written).unwrap();
        println!("Request sent:\n{}", request);
    }

    // In production, you would read the response from the connection
    // use std::io::Read;
    // let mut response = String::new();
    // conn.read_to_string(&mut response)?;

    Ok(())
}