use std::{ fs::{self, read_to_string}, path::PathBuf, process::{exit, Command}, sync::Mutex };
use base64::{Engine as _, engine::general_purpose};
use tokio::time::{interval, Duration};
use once_cell::sync::Lazy;
use image::DynamicImage;
use serde_json::Value;
use reqwest::Client;

static KEYS: Lazy<Mutex<Value>> = Lazy::new(|| Mutex::new(Value::Array(vec![])));
static KI: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
static SONGS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));
static TOKEN: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static CURRENT: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

pub async fn run() {
    let home_dir = std::env::var("HOME").unwrap();
    let output_dir = PathBuf::from(&home_dir).join(".images");
    fs::create_dir_all(&output_dir).unwrap();

    let path = PathBuf::from(&home_dir).join(".images");
    std::fs::create_dir_all(&path).unwrap();

    if !std::fs::exists(&path.join("config.json")).unwrap() {
        std::fs::write(&path.join("config.json"), "[]").unwrap();
        println!("No keys found, please run again with the --add-key flag");
        exit(0)
    }

    *KEYS.lock().unwrap() = serde_json::from_str(
        &read_to_string(&path.join("config.json")).unwrap()
    ).unwrap();

    if KEYS.lock().unwrap().as_array().unwrap().len() == 0 {
        println!("No keys found, please run again with the --add-key flag");
        exit(0)
    }

    *SONGS.lock().unwrap() = fs::read_dir(&output_dir).unwrap().map(|v| 
        v
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .replace(".png", "")
        ).collect();

    let refresh_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(35 * 60));
        loop {
            refresh_token().await;
            interval.tick().await;
        }
    });

    let keep_handle = tokio::spawn( async move {
        loop { get_img_link(&output_dir).await }
    });

    tokio::try_join!(refresh_handle, keep_handle).ok();
}

async fn get_img_link(output_dir: &PathBuf) {
    let tkn = &*TOKEN.lock().unwrap().to_owned();
    let mut response = None;
    if let Ok(res) = Client::new()
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .bearer_auth(tkn)
        .send()
        .await 
    { response = Some(res) }

    if let Some(res) = response {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(image_url) = json.pointer("/item/album/images/1/url").and_then(|val| val.as_str()) {
                    if let Some(name) = json.pointer("/item/album/name").and_then(|val| val.as_str()) {
                        match  show(&image_url, &output_dir, name, 1024, 728).await {
                            Ok(_) => {},
                            Err(_) => *CURRENT.lock().unwrap() = "".to_string()
                            
                        }
                    }
                }
            }
        } else if res.status().as_u16() == 429 {
            *KI.lock().unwrap() += 1;
            if *KI.lock().unwrap() > KEYS.lock().unwrap().as_array().unwrap().len() - 1 { *KI.lock().unwrap() = 0 }
            refresh_token().await;
        }
    }
}

async fn show(img_url: &str, output_dir: &PathBuf, name: &str, screen_width: u32, screen_height: u32) -> Result<bool, Box<dyn std::error::Error>> {
    if &*CURRENT.lock().unwrap().as_str() == name { return Ok(false) }
    *CURRENT.lock().unwrap() = name.to_string();

    let output_path= output_dir.join(format!("{}.png", name));
    for song in SONGS.lock().unwrap().clone() {
        if song == name {
            Command::new("wal").args(&["-qeti", output_path.to_str().unwrap()]).output()?;
            println!("( cache ) {}", name);
            return  Ok(true)
        }
    }

    let response = reqwest::get(img_url).await?;
    let bytes = response.bytes().await?;
    let original_image = image::load_from_memory(&bytes)?;
    let resized_image = original_image.resize_exact(screen_width, screen_height, image::imageops::FilterType::Lanczos3);
    let blurred_image = resized_image.blur(15.0);

    let mut canvas = DynamicImage::new_rgb8(screen_width, screen_height);
    image::imageops::overlay(&mut canvas, &blurred_image, 0, 0);
    image::imageops::overlay(
        &mut canvas,
        &original_image,
        (screen_width / 2 - original_image.width() / 2) as i64,
        (screen_height / 2 - original_image.height() / 2) as i64,
    );

    canvas.save(&output_path)?;
    SONGS.lock().unwrap().push(name.to_string());

    Command::new("wal").args(&["-qeti", output_path.to_str().unwrap()]).output()?;

    println!("( new ) {}", name);
    Ok(true)
}

async fn refresh_token() {
    let i = KI.lock().unwrap().to_owned();
    let keys = KEYS.lock().unwrap().as_array().unwrap()[i].clone();
    let credentials = general_purpose::STANDARD.encode(format!("{}:{}", keys["id"].as_str().unwrap(), keys["secret"].as_str().unwrap()));
    let params = [ ("grant_type", "refresh_token"), ("refresh_token", keys["token"].as_str().unwrap())];

    let response = Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", credentials))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send().await;

    match response {
        Ok(res) if res.status().is_success() => {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(access_token) = json["access_token"].as_str() {
                    *TOKEN.lock().unwrap() = access_token.to_string();
                }
            }
        }
        _ => Box::pin(refresh_token()).await
    }
}