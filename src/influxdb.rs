extern crate reqwest;

use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

pub fn influx_new_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer fUqLk2fqqb62jyE2PYNpx1mNbu38s75SKN7thO1nKpNqf2vRzb24QWopAlUjh-WM54xJ2KJA2_jXDYzGSlPKDQ=="));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    return Client::builder().default_headers(headers).build().unwrap();
}

pub fn write(client: &Client, data: String) {
    let url: &str = "http://192.168.2.100:8086/api/v2/write?bucket=default&org=SmartHome";
    let req = client.post(url).body(data).send();
    tokio::spawn(async move {
        assert!(req.await.is_ok(), "Errorrr when writing to influxdb");
    });
}
