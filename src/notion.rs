use reqwest::{blocking::Client, header::HeaderMap};
use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
pub struct PropOption {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    // Most of these are not properly implemented because I'm lazy
    Checkbox {},
    CreatedBy {},
    CreatedTime {},
    Date {/* Empty */},
    Email {},
    Files {},
    Formula {},
    LastEditedBy {},
    LastEditedTime {},
    MultiSelect {},
    Number {},
    People {},
    PhoneNumber {},
    Relation {},
    RichText {},
    Rollup {},
    Select { options: Vec<PropOption> },
    Status { options: Vec<PropOption> },
    Title {/* Empty */},
    Url {},
}

#[derive(serde::Deserialize, Debug)]
pub struct Property {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub inner: PropertyType,
}

#[derive(serde::Deserialize, Debug)]
pub struct Database {
    pub id: String,
    pub properties: HashMap<String, Property>,
}

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
        headers.insert("Notion-Version", "2022-06-28".parse().unwrap());
        NotionClient {
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub fn database(self: &Self, id: String) -> Database {
        let url = format!("https://api.notion.com/v1/databases/{}", id);
        let resp = self
            .client
            .get(url)
            .send()
            .expect("Couldn't fetch database from API")
            .text()
            .unwrap();
        println!("{}", resp);
        serde_json::from_str(resp.as_str()).expect("Couldn't parse database API response")
    }
}
