use chrono::{DateTime, NaiveDate, Utc};
use reqwest::{blocking::Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct PropOption {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct PropertyType {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub inner: PropertyTypeInner,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub id: String,
    pub properties: HashMap<String, PropertyType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateValue {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextValue {
    pub content: String,
    pub link: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TitleValue {
    pub text: TextValue,
    #[serde(skip_serializing)]
    pub plain_text: String,
}

impl TitleValue {
    pub fn new(text: impl Into<String>, link: Option<String>) -> TitleValue {
        TitleValue {
            text: TextValue {
                content: text.into(),
                link: link,
            },
            plain_text: String::new(),
        }
    }
}

// Both have the same attributes
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusSelectValue {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub color: String,
}

impl StatusSelectValue {
    pub fn new(name: impl Into<String>) -> StatusSelectValue {
        StatusSelectValue {
            id: String::new(),
            name: name.into(),
            color: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
    Number(Option<u32>),
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

#[derive(Serialize, Deserialize, Debug)]
pub struct PropertyValue {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub inner: PropertyValueInner,
}

#[derive(Deserialize, Debug)]
pub struct Page {
    pub id: String,
    pub properties: HashMap<String, PropertyValue>,
}

pub struct NotionClient {
    client: Client,
}

#[derive(Serialize, Debug)]
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FilterJoin {
    And(Vec<Filter>),
    Or(Vec<Filter>),
}

#[derive(Serialize, Debug)]
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
        let body = serde_json::to_string(&filter).unwrap();
        #[derive(Deserialize, Debug)]
        struct QueryResponse {
            results: Vec<Page>,
            next_cursor: Option<String>,
            has_more: bool,
            // There's more properties but I am LAZY
        }

        let resp = self
            .client
            .post(url)
            .body(body)
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

    pub fn create_page(
        &self,
        database: &Database,
        properties: HashMap<&str, PropertyValueInner>,
    ) -> Page {
        let properties = serde_json::json!({
            "parent": {
                "database_id": database.id
            },
            "properties": properties
        });
        let body = serde_json::to_string(&properties).unwrap();
        let resp = self
            .client
            .post("https://api.notion.com/v1/pages")
            .body(body)
            .send()
            .expect("Couldn't create page")
            .text()
            .unwrap();
        serde_json::from_str(resp.as_str()).unwrap()
    }

    pub fn update_page(&self, page: &Page, properties: HashMap<&str, PropertyValueInner>) -> Page {
        let properties = serde_json::json!({
            "properties": properties
        });
        let url = format!("https://api.notion.com/v1/pages/{}", page.id);
        let body = serde_json::to_string(&properties).unwrap();
        let resp = self
            .client
            .patch(url)
            .body(body)
            .send()
            .expect("Couldn't update page")
            .text()
            .unwrap();
        serde_json::from_str(resp.as_str()).unwrap()
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
    fn query_resp_deser() {
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

    #[test]
    fn property_ser() {
        let props = HashMap::from([
            (
                "name",
                PropertyValueInner::Title(vec![TitleValue::new("testing lsdkfjdslkf", None)]),
            ),
            (
                "subject",
                PropertyValueInner::Select(StatusSelectValue::new("subject 2")),
            ),
        ]);
        let str = serde_json::to_string_pretty(&serde_json::json!({
            "properties": props
        }))
        .unwrap();
        println!("{}", str);
    }

    #[test]
    fn create_page_json() {
        let db = Database {
            id: "fake database id".into(),
            properties: HashMap::new(),
        };
        let properties = HashMap::from([
            ("id", PropertyValueInner::Number(Some(3333))),
            (
                "subject",
                PropertyValueInner::Select(StatusSelectValue::new("subject 2")),
            ),
        ]);
        let properties = serde_json::json!({
            "parent": {
                "database_id": db.id
            },
            "properties": properties
        });
        let str = serde_json::to_string_pretty(&properties).unwrap();
        println!("{}", str);
    }

    #[test]
    fn update_page_json() {
        let properties = HashMap::from([
            (
                "name",
                PropertyValueInner::Title(vec![TitleValue::new("test title", None)]),
            ),
            (
                "due",
                PropertyValueInner::Date(Some(DateValue {
                    start: chrono::Local::now().to_utc(),
                    end: None,
                })),
            ),
        ]);
        let properties = serde_json::json!({
            "properties": properties
        });
        let str = serde_json::to_string_pretty(&properties).unwrap();
        println!("{}", str);
    }
}
