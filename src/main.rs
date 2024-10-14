mod canvas;
mod notion;

use canvas::CanvasClient;
use notion::{Filter, FilterJoin, FilterMatch, NotionClient, PropertyTypeInner};
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct DatabaseConfig {
    id: String,
    // A map of Canvas course IDs to Notion subject names.
    alias: HashMap<u32, String>,
}

#[derive(serde::Deserialize)]
struct NotionConfig {
    token: String,
    database: DatabaseConfig,
}

#[derive(serde::Deserialize)]
struct Config {
    canvas_token: String,
    notion: NotionConfig,
}

fn main() {
    let config = std::fs::read_to_string("config.json").expect("Couldn't read config file");
    let config: Config = serde_json::from_str(config.as_str()).expect("Couldn't parse config file");

    let canvas = CanvasClient::new(config.canvas_token);
    let notion = NotionClient::new(config.notion.token);
    let db = notion.database(config.notion.database.id);

    let status = db
        .properties
        .values()
        .find(|prop| matches!(&prop.inner, PropertyTypeInner::Status { options: _ }))
        .expect("Couldn't find status property");
    let subject = db
        .properties
        .values()
        .find(|prop| matches!(&prop.inner, PropertyTypeInner::Select { options: _ }))
        .expect("Couldn't find subject property");

    println!("Status: {:?}, subject: {:?}", status, subject);

    let courses: Vec<canvas::Course> = canvas
        .courses()
        .into_iter()
        .filter(|course| config.notion.database.alias.contains_key(&course.id))
        .collect();

    for course in courses {
        let subject = config
            .notion
            .database
            .alias
            .get(&course.id)
            .expect("We found this in a map already so this should always work");
        let filter = Filter::Match {
            // TODO: This is entirely hardcoded.
            property: "subject".into(),
            inner: FilterMatch::Select {
                equals: subject.into(),
            },
        };
        let pages = notion.query(&db, filter);
        println!("{:?}", pages);
        /*
        println!("{:?} ---", course);
        for assignment in canvas.assignments(&course) {
            println!("{:?}", assignment);
        }
        */
    }
}
