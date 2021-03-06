use std::process::Command;
use serde_json::Value;
use clap::{Arg, App};
use reqwest::header::USER_AGENT;

struct PR {
    repo: String,
    pull: String,
}

#[tokio::main]
async fn search(user: &String, client: &reqwest::Client) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/search/issues?q=author:{}+type:pr+is:open", user);
    let payload = client.get(url.as_str())
        .header(USER_AGENT, "My Rust Program 1.0")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(payload)
}

fn parse(search: &Value) -> Vec<PR> {
    let items = search["items"].as_array().unwrap();
    let mut vec = Vec::new();
    for item in items {
        let repository_url = item["repository_url"].as_str().unwrap();
        let number = item["number"].as_u64().unwrap();
        vec.push(PR{
            repo: String::from(repository_url),
            pull: format!("{}/pulls/{}", repository_url, number)
        })
    }
    vec
}

fn get_pr_sha(pr: &PR) -> Option<String> {
    let out = match Command::new("gh")
        .arg("api")
        .arg(&pr.pull)
        .output() {
        Ok(k) => k.stdout,
        Err(e) => panic!("Command failed: {}", e),
    };
    let str_out = match String::from_utf8(out) {
        Ok(k) => String::from(k.as_str()),
        Err(e) => panic!("String loading failed: {}", e),
    };
    let payload: Value = match serde_json::from_str(str_out.as_str()) {
        Ok(k) => k,
        Err(e) => panic!("Parsing failed: {}", e),
    };
    match payload["head"].get("sha") {
        Some(sha) => Some(String::from(sha.as_str().unwrap())),
        None => None,
    }
}

fn get_pr_status(pr: &PR, sha: &String) -> Option<String> {
    let out = match Command::new("gh")
        .arg("api")
        .arg(format!("{}/commits/{}/status", pr.repo, sha))
        .output() {
        Ok(k) => k.stdout,
        Err(e) => panic!("Command failed: {}", e),
    };
    let str_out = match String::from_utf8(out) {
        Ok(k) => String::from(k.as_str()),
        Err(e) => panic!("String loading failed: {}", e),
    };
    let payload: Value = match serde_json::from_str(str_out.as_str()) {
        Ok(k) => k,
        Err(e) => panic!("Parsing failed: {}", e),
    };
    match payload.get("state") {
        Some(state) => Some(String::from(state.as_str().unwrap())),
        None => None,
    }
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
    let prs = parse(&res);
    for pr in prs {
        if let Some(sha) = get_pr_sha(&pr){
            if let Some(status) = get_pr_status(&pr, &sha){
                println!("{} {} {}", pr.repo, sha, status);
            }
        }
    }
}
