use chrono::{DateTime, NaiveDate, Utc};
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
pub enum PropertyTypeInner {
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
pub struct PropertyType {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub inner: PropertyTypeInner,
}

#[derive(serde::Deserialize, Debug)]
pub struct Database {
    pub id: String,
    pub properties: HashMap<String, PropertyType>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DateValue {
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TitleValue {
    plain_text: String,
}

// Both have the same attributes
#[derive(serde::Deserialize, Debug)]
pub struct StatusSelectValue {
    id: String,
    name: String,
    color: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PropertyValueInner {
    // Most of these are not properly implemented because I'm lazy
    Checkbox {},
    CreatedBy {},
    CreatedTime {},
    Date(Option<DateValue>),
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
    Select(StatusSelectValue),
    Status(StatusSelectValue),
    Title(Vec<TitleValue>),
    Url {},
}

#[derive(serde::Deserialize, Debug)]
pub struct PropertyValue {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub inner: PropertyValueInner,
}

#[derive(serde::Deserialize, Debug)]
pub struct Page {
    pub id: String,
    pub properties: HashMap<String, PropertyValue>,
}

pub struct NotionClient {
    client: Client,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FilterMatch {
    // Most of these are not properly implemented because I'm lazy
    Checkbox {},
    Date { equals: NaiveDate },
    Files {},
    Formula {},
    MultiSelect {},
    Number {},
    People {},
    PhoneNumber {},
    Relation {},
    RichText {},
    Select { equals: String },
    Status {},
    Timestamp {},
    ID {},
    Title { equals: String },
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FilterJoin {
    And(Vec<Filter>),
    Or(Vec<Filter>),
}

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub enum Filter {
    Join(FilterJoin),
    Match {
        property: String,
        #[serde(flatten)]
        inner: FilterMatch,
    },
}

impl NotionClient {
    pub fn new(token: String) -> NotionClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        headers.insert("Notion-Version", "2022-06-28".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        NotionClient {
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub fn database(&self, id: String) -> Database {
        let url = format!("https://api.notion.com/v1/databases/{}", id);
        let resp = self
            .client
            .get(url)
            .send()
            .expect("Couldn't fetch database from API")
            .text()
            .unwrap();
        // println!("{}", resp);
        serde_json::from_str(resp.as_str()).expect("Couldn't parse database API response")
    }

    // TODO: Support filters
    pub fn query(&self, database: &Database, filter: Filter) -> Vec<Page> {
        let url = format!("https://api.notion.com/v1/databases/{}/query", database.id);
        let filter = serde_json::json!({
            "filter": filter
        });
        let string = serde_json::to_string(&filter).unwrap();
        #[derive(serde::Deserialize, Debug)]
        struct QueryResponse {
            results: Vec<Page>,
            next_cursor: Option<String>,
            has_more: bool,
            // There's more properties but I am LAZY
        }

        let resp = self
            .client
            .post(url)
            .body(string)
            .send()
            .expect("Couldn't query database from API")
            .text()
            .unwrap();

        let obj: QueryResponse = serde_json::from_str(resp.as_str())
            .expect("Couldn't parse database query API response");

        if obj.has_more {
            println!("WARNING: There are more query results that I am too lazy to fetch");
        }

        obj.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_serialize() {
        let filter = serde_json::json!({
            "filter": Filter::Join(FilterJoin::And(vec![
                Filter::Match {
                    property: "name".into(),
                    inner: FilterMatch::Title {
                        equals: "test name".into(),
                    },
                },
                Filter::Match {
                    property: "subject".into(),
                    inner: FilterMatch::Select {
                        equals: "my class".into(),
                    },
                },
            ]))
        });
        let string = serde_json::to_string_pretty(&filter).unwrap();
        println!("{}", string);
    }

    #[test]
    fn query_resp_fake_deser() {
        let str = r#"
            [{
                "object": "page",
                "id": "lskdjfsldkfjslkjfdlskfjslkdf",
                "properties": {
                    "status": {
                        "id": "AAAA",
                        "type": "status",
                        "status": {
                            "id": "lskdfjsdlkfjsdfldkjfldfjkl",
                            "name": "planned",
                            "color": "default"
                        }
                    }
                }
            }]"#;
        let pages: Vec<Page> = serde_json::from_str(str).unwrap();
        println!("{:?}", pages);
    }
}
