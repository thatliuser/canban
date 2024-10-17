use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::HeaderMap,
    Url,
};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Submitted,
    Unsubmitted,
    Graded,
    PendingReview,
}

#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub enrollment_term_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub id: u32,
    pub workflow_state: WorkflowState,
}

#[derive(Deserialize, Debug)]
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

    fn send<T: DeserializeOwned>(
        &self,
        path: impl Into<String>,
        method: fn(&Client, Url) -> RequestBuilder,
    ) -> Result<T> {
        let path: String = path.into();
        let url = Url::parse(&format!("https://{}/api/v1/{}", self.base, path))
            .with_context(|| format!("Couldn't parse URL with path {}", path))?;
        let resp = method(&self.client, url)
            .send()
            .with_context(|| format!("Couldn't make request to path {}", path))?
            .error_for_status()
            .with_context(|| format!("Non-200 status returned from path {}", path))?
            .text()
            .with_context(|| format!("Couldn't convert response from path {}", path))?;
        serde_json::from_str(&resp)
            .with_context(|| format!("Couldn't parse API response from path {}", path))
    }

    pub fn courses(&self) -> Result<Vec<Course>> {
        self.send("courses?page=1&per_page=100", Client::get)
    }

    pub fn assignments(&self, course: &Course) -> Result<Vec<Assignment>> {
        self.send(
            format!("courses/{}/assignments?include=submission", course.id),
            Client::get,
        )
    }
}
