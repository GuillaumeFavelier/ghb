use std::process::Command;
use serde_json::Value;
use clap::{Arg, App};

struct PR {
    repo: String,
    pull: String,
}

fn get_prs(user: &String) -> Vec<PR> {
    // https://api.github.com/search/issues?q=author:GuillaumeFavelier+type:pr+is:open
    let out = match Command::new("gh")
        .arg("api")
        .arg("-f")
        .arg(format!("q=author:{} type:pr is:open", user))
        .arg("-X")
        .arg("GET")
        .arg("search/issues")
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
    let mut vec = Vec::new();
    if let Some(items) = &payload["items"].as_array() {
        for item in items.iter() {
            let repository_url = item["repository_url"].as_str().unwrap();
            let number = item["number"].as_u64().unwrap();
            vec.push(PR{
                repo: String::from(repository_url),
                pull: format!("{}/pulls/{}", repository_url, number)
            })
        }
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
    let prs = get_prs(&user);
    for pr in prs {
        if let Some(sha) = get_pr_sha(&pr){
            if let Some(status) = get_pr_status(&pr, &sha){
                println!("{} {} {}", pr.repo, sha, status);
            }
        }
    }
}
