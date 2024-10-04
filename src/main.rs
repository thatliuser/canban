mod canvas;
mod notion;

#[derive(serde::Deserialize)]
struct DatabaseConfig {
    id: String,
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

    let canvas = canvas::CanvasClient::new(config.canvas_token);
    let notion = notion::NotionClient::new(config.notion.token);

    /*
    let courses: Vec<canvas::Course> = canvas
        .courses()
        .into_iter()
        .filter(|course| course.enrollment_term_id == 354)
        .collect();

    for course in courses {
        println!("{:?} ---", course);
        for assignment in canvas.assignments(&course) {
            println!("{:?}", assignment);
        }
    }
    */

    let db = notion.database(config.notion.database.id);
    println!("{:?}", db);
    for (key, prop) in db.properties {
        match key.as_str() {
            "status" => {}
            "subject" => {}
            _ => {}
        }
        match prop.inner {
            notion::PropertyType::Select { options } => {}
            notion::PropertyType::Status { options } => {}
            _ => {}
        }
    }
}
