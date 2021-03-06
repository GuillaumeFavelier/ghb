use serde_json::Value;
use reqwest::header::USER_AGENT;
use read_input::prelude::*;

struct Auth {
    client: reqwest::Client,
    login: String,
    token: String,
}

impl Auth {
    fn new() -> Auth {
        Auth{
            client: reqwest::Client::new(),
            login: String::new(),
            token: String::new(),
        }
    }
}

#[tokio::main]
async fn search(auth: &Auth) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/search/issues?q=author:{}+type:pr+is:open", auth.login);
    let payload = auth.client.get(url.as_str())
        .header(USER_AGENT, "GitHub Board")
        .bearer_auth(auth.token.as_str())
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(payload)
}

#[tokio::main]
async fn get_pr_sha(pr: &Value, auth: &Auth) -> Result<Value, Box<dyn std::error::Error>> {
    let repository_url = pr["repository_url"].as_str().unwrap();
    let number = pr["number"].as_u64().unwrap();
    let url = format!("{}/pulls/{}", repository_url, number);
    let payload = auth.client.get(url.as_str())
        .header(USER_AGENT, "My Rust Program 1.0")
        .bearer_auth(auth.token.as_str())
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(payload)
}

#[tokio::main]
async fn get_pr_status(pr: &Value, sha: &Value, auth: &Auth) -> Result<Value, Box<dyn std::error::Error>> {
    let repository_url = pr["repository_url"].as_str().unwrap();
    let commit = sha["head"]["sha"].as_str().unwrap();
    let url = format!("{}/commits/{}/status", repository_url, commit);
    let payload = auth.client.get(url.as_str())
        .header(USER_AGENT, "My Rust Program 1.0")
        .bearer_auth(auth.token.as_str())
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(payload)
}

fn main() {
    let mut auth = Auth::new();
    auth.login = input().msg("login: ").get();
    auth.token = rpassword::prompt_password_stdout("token: ").unwrap();
    let res = match search(&auth) {
        Ok(s) => s,
        Err(e) => panic!("Fail to search: {}", e),
    };
    let prs = res["items"].as_array().unwrap();
    for pr in prs {
        let sha = match get_pr_sha(&pr, &auth){
            Ok(s) => s,
            Err(e) => panic!("Fail to fetch sha: {}", e),
        };
        match get_pr_status(&pr, &sha, &auth){
            Ok(s) => println!("{} {}", pr["url"], s["state"]),
            Err(e) => panic!("Fail to fetch sha: {}", e),
        };
    }
}
