use dotenvy::dotenv;
use reqwest::Client;

fn main() {
    dotenv().unwrap();
    let (key, token) = std::env::vars().find(|(key, val)| key == "TOKEN").unwrap();
    println!("{}", token);
    // let mut client = Client::new();
}
