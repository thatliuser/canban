use reqwest::{blocking::Client, header::HeaderMap};

pub struct NotionClient {
    client: Client,
}

impl NotionClient {
    pub fn new(token: String) -> NotionClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        NotionClient {
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }
}
