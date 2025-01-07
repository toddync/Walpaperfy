use std::{fs::read_to_string, path::PathBuf, process::exit};
use base64::{Engine as _, engine::general_purpose};
use cli_prompts_rs::{CliPrompt, LogType};
use serde_json::{Map, Value};
use reqwest::Client;

pub async fn run() {
    let path = PathBuf::from(std::env::var("HOME").unwrap()).join(".images");
    std::fs::create_dir_all(&path).unwrap();

    if !std::fs::exists(&path.join("config.json")).unwrap() {
        std::fs::write(&path.join("config.json"), "[]").unwrap();
    }

    let mut config: Value = serde_json::from_str(
        &read_to_string(&path.join("config.json")).unwrap()
    ).unwrap();

    let mut cli_prompt = CliPrompt::new();

    let id = cli_prompt.prompt_text("Enter your Client Id").unwrap();
    if id == "" { cli_prompt.cancel("Invalid Client Id").unwrap(); exit(0) }

    let secret = cli_prompt.prompt_text("Enter your Client Secret").unwrap();
    if secret == "" { cli_prompt.cancel("Invalid Client Secret").unwrap(); exit(0) }

    let mut uri = cli_prompt.prompt_text(r#"Enter your Redirect URI (Default: "http://localhost")"#).unwrap();
    if uri == "" { uri = format!("http://localhost") }
    else { uri = uri.trim_end_matches("/").to_string() }

    let scope = "user-read-currently-playing";
    cli_prompt
        .log(&format!("go to: https://accounts.spotify.com/authorize?response_type=code&client_id={id}&scope={scope}&redirect_uri={uri}"), LogType::Info)
        .unwrap();

    let mut code = cli_prompt.prompt_text("Enter the url you got redirected to").unwrap();
    if code == "" { cli_prompt.cancel("Invalid Code").unwrap(); exit(0); }
    else { code = code.replace(&format!("{uri}/?code="), ""); }


    let params = [ 
        ("grant_type", "authorization_code"),
        ("redirect_uri", &uri),
        ("code", &code)
    ];

    let credentials = general_purpose::STANDARD.encode(format!("{}:{}", id, secret));
    let response = Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", credentials))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send().await;

    let mut token = "".to_string();
    if let Ok(res) = response {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(tkn) = json.clone().pointer("/refresh_token").and_then(|val| val.as_str()) {
                    token = format!("{tkn}");
                }
            }
        } else { cli_prompt.cancel("Error getting token").unwrap(); exit(0) }
    }

    let mut to_add = Map::new();

    to_add.insert(format!("id"), Value::String(format!("{id}")));
    to_add.insert(format!("secret"), Value::String(format!("{secret}")));
    to_add.insert(format!("token"), Value::String(format!("{token}")));
    config.as_array_mut().unwrap().push(Value::Object(to_add));
    std::fs::write(&path.join("config.json"), config.to_string()).unwrap();

    cli_prompt.outro("Information saved :)").unwrap();
}