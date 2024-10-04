use chrono::{DateTime, Utc};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum WorkflowState {
    Submitted,
    Unsubmitted,
    Graded,
    PendingReview,
}

#[derive(serde::Deserialize, Debug)]
struct Course {
    id: u32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    enrollment_term_id: u32,
}

#[derive(serde::Deserialize, Debug)]
struct Submission {
    id: u32,
    workflow_state: WorkflowState,
}

#[derive(serde::Deserialize, Debug)]
struct Assignment {
    id: u32,
    name: String,
    due_at: Option<DateTime<Utc>>,
    html_url: String,
    submission: Option<Submission>,
}

fn main() {
    dotenvy::dotenv().expect("Couldn't load .env file; did you create it?");
    let (_, token) = std::env::vars()
        .find(|(key, _)| key == "TOKEN")
        .expect("Couldn't find TOKEN in .env file; did you set it?");
    let token = format!("Bearer {}", token);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://canvas.eee.uci.edu/api/v1/courses?page=1&per_page=100")
        .header("Authorization", token.clone())
        .send()
        .expect("Couldn't fetch courses from API")
        .text()
        .unwrap();
    let all_courses: Vec<Course> =
        serde_json::from_str(resp.as_str()).expect("Couldn't parse course API response");

    let curr_courses: Vec<Course> = all_courses
        .into_iter()
        .filter(|course| course.enrollment_term_id == 354)
        .collect();

    for course in curr_courses {
        println!("Course {:?}: ", course);
        let url = format!(
            "https://canvas.eee.uci.edu/api/v1/courses/{}/assignments?include=submission",
            course.id
        );
        let resp = client
            .get(url)
            .header("Authorization", token.clone())
            .send()
            .expect("Couldn't fetch assignments from API")
            .text()
            .unwrap();

        let assignments: Vec<Assignment> =
            serde_json::from_str(resp.as_str()).expect("Couldn't parse assignment API response");

        for assignment in assignments {
            println!("{:?}", assignment);
        }
    }
}
