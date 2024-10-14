use chrono::{DateTime, Utc};
use reqwest::{blocking::Client, header::HeaderMap};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Submitted,
    Unsubmitted,
    Graded,
    PendingReview,
}

#[derive(serde::Deserialize, Debug)]
pub struct Course {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub enrollment_term_id: u32,
}

#[derive(serde::Deserialize, Debug)]
pub struct Submission {
    pub id: u32,
    pub workflow_state: WorkflowState,
}

#[derive(serde::Deserialize, Debug)]
pub struct Assignment {
    pub id: u32,
    pub name: String,
    pub due_at: Option<DateTime<Utc>>,
    pub html_url: String,
    pub submission: Option<Submission>,
}

pub struct CanvasClient {
    client: Client,
    base: String,
}

impl CanvasClient {
    pub fn new(base: String, token: String) -> CanvasClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        CanvasClient {
            client: Client::builder().default_headers(headers).build().unwrap(),
            base: base,
        }
    }

    pub fn courses(&self) -> Vec<Course> {
        let url = format!("https://{}/api/v1/courses?page=1&per_page=100", self.base);
        let resp = self
            .client
            .get(url)
            .send()
            .expect("Couldn't fetch courses from API")
            .text()
            .unwrap();
        serde_json::from_str(resp.as_str()).expect("Couldn't parse course API response")
    }

    pub fn assignments(&self, course: &Course) -> Vec<Assignment> {
        let url = format!(
            "https://{}/api/v1/courses/{}/assignments?include=submission",
            self.base, course.id
        );
        let resp = self
            .client
            .get(url)
            .send()
            .expect("Couldn't fetch assignments from API")
            .text()
            .unwrap();

        serde_json::from_str(resp.as_str()).expect("Couldn't parse assignment API response")
    }
}
