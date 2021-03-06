use serde_json::Value;
use clap::{Arg, App};
use reqwest::header::USER_AGENT;

#[tokio::main]
async fn search(user: &String, client: &reqwest::Client) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/search/issues?q=author:{}+type:pr+is:open", user);
    let payload = client.get(url.as_str())
        .header(USER_AGENT, "GitHub Board")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(payload)
}

#[tokio::main]
async fn get_pr_sha(pr: &Value, client: &reqwest::Client) -> Result<Value, Box<dyn std::error::Error>> {
    let repository_url = pr["repository_url"].as_str().unwrap();
    let number = pr["number"].as_u64().unwrap();
    let url = format!("{}/pulls/{}", repository_url, number);
    let payload = client.get(url.as_str())
        .header(USER_AGENT, "My Rust Program 1.0")
        .send()
        .await?
        .json::<Value>()
        .await?;
    println!("{}", payload);
    Ok(payload)
}

#[tokio::main]
async fn get_pr_status(pr: &Value, sha: &Value, client: &reqwest::Client) -> Result<Value, Box<dyn std::error::Error>> {
    let repository_url = pr["repository_url"].as_str().unwrap();
    let commit = sha["head"]["sha"].as_str().unwrap();
    let url = format!("{}/commits/{}/status", repository_url, commit);
    let payload = client.get(url.as_str())
        .header(USER_AGENT, "My Rust Program 1.0")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(payload)
}

fn main() {
    let matches = App::new("GitHub Board")
            .arg(Arg::new("user")
                .short('u')
                .long("user")
                .value_name("USER")
                .about("Specify GitHub user login")
                .required(true))
        .get_matches();
    let user = match matches.value_of("user") {
        Some(k) => String::from(k),
        None => panic!("User login not provided"),
    };
    let client = reqwest::Client::new();
    let res = match search(&user, &client) {
        Ok(s) => s,
        Err(e) => panic!("Fail to search: {}", e),
    };
    let prs = res["items"].as_array().unwrap();
    for pr in prs {
        let sha = match get_pr_sha(&pr, &client){
            Ok(s) => s,
            Err(e) => panic!("Fail to fetch sha: {}", e),
        };
        match get_pr_status(&pr, &sha, &client){
            Ok(s) => println!("{} {}", pr["repository_url"], s["state"]),
            Err(e) => panic!("Fail to fetch sha: {}", e),
        };
    }
}
