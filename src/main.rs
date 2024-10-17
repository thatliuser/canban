mod canvas;
mod notion;

use canvas::{Assignment, CanvasClient};
use chrono::{DateTime, DurationRound, Local, TimeDelta};
use colored::Colorize;
use notion::{
    DateValue, Filter, FilterJoin, FilterMatch, NotionClient, Page, PropertyTypeInner,
    PropertyValueInner, StatusSelectValue, TitleValue,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct DatabaseConfig {
    id: String,
    // A map of Canvas course IDs to Notion subject names.
    alias: HashMap<u32, String>,
}

#[derive(Deserialize)]
struct NotionConfig {
    token: String,
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct CanvasConfig {
    token: String,
    base_url: String,
}

#[derive(Deserialize)]
struct Config {
    canvas: CanvasConfig,
    notion: NotionConfig,
}

fn needs_name_update(page: &Page, assignment: &Assignment) -> bool {
    page.properties
        .get("name")
        .and_then(|name| match &name.inner {
            PropertyValueInner::Title(value) => value.get(0),
            _ => None,
        })
        .map(|name| &name.text.content)
        != Some(&assignment.name)
}

fn needs_due_update(page: &Page, assignment: &Assignment) -> bool {
    // Clone the date and convert the timezone
    let due: Option<DateTime<Local>> = assignment.due_at.map(|due| {
        // Truncate the duration because Notion pages don't accept seconds
        due.clone()
            .duration_trunc(TimeDelta::minutes(1))
            .unwrap()
            .into()
    });
    page.properties
        .get("due")
        .cloned()
        .and_then(|due| match due.inner {
            PropertyValueInner::Date(date) => date,
            _ => None,
        })
        .map(|page| page.start)
        != due
}

fn needs_update(page: &Page, assignment: &Assignment) -> bool {
    needs_due_update(page, assignment) || needs_name_update(page, assignment)
}

fn main() {
    let config = std::fs::read_to_string("config.json").expect("Couldn't read config file");
    let config: Config = serde_json::from_str(config.as_str()).expect("Couldn't parse config file");

    let canvas = CanvasClient::new(config.canvas.base_url, config.canvas.token);
    let notion = NotionClient::new(config.notion.token);
    let db = notion.database(config.notion.database.id);

    let courses: Vec<canvas::Course> = canvas
        .courses()
        .into_iter()
        .filter(|course| config.notion.database.alias.contains_key(&course.id))
        .collect();

    for course in courses {
        println!(
            "Iterating course {} (id {})",
            course.name.blue(),
            course.id.to_string().cyan()
        );
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
        let mut pages: HashMap<u32, Page> = notion
            .query(&db, filter)
            .into_iter()
            .filter_map(|page| {
                let id = page.properties.get("id")?;
                let PropertyValueInner::Number(id) = id.inner else {
                    return None;
                };
                Some((id?, page))
            })
            .collect();
        for assignment in canvas.assignments(&course) {
            println!("> Assignment '{}':", assignment.name.magenta());
            let page = pages.remove(&assignment.id).unwrap_or_else(|| {
                println!("{}", ">> No page found, creating new page".red());
                notion.create_page(
                    &db,
                    HashMap::from([
                        ("id", PropertyValueInner::Number(Some(assignment.id))),
                        (
                            "subject",
                            PropertyValueInner::Select(StatusSelectValue::new(subject)),
                        ),
                    ]),
                )
            });
            if !needs_update(&page, &assignment) {
                println!("{}", ">> Assignment up to date, skipping".green());
                continue;
            }
            // Update the properties that don't exist
            println!("{}", ">> Updating assignment name and due date".yellow());
            notion.update_page(
                &page,
                HashMap::from([
                    (
                        "name",
                        PropertyValueInner::Title(vec![TitleValue::new(assignment.name, None)]),
                    ),
                    (
                        "due",
                        PropertyValueInner::Date(assignment.due_at.map(|date| DateValue {
                            start: date.into(),
                            end: None,
                        })),
                    ),
                ]),
            );
        }
    }
}
