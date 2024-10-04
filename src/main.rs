mod canvas;
mod notion;

#[derive(serde::Deserialize)]
struct Config {
    canvas_token: String,
    notion_token: String,
}

fn main() {
    let config = std::fs::read_to_string("config.json").expect("Couldn't read config file");
    let config: Config = serde_json::from_str(config.as_str()).expect("Couldn't parse config file");

    let canvas = canvas::CanvasClient::new(config.canvas_token);
    let notion = notion::NotionClient::new(config.notion_token);

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
}
