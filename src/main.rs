use std::{
    fs, path::PathBuf, process::Command, sync::Mutex
};

mod env;
use env::KEYS;

use base64::{Engine as _, engine::general_purpose};
use imageproc::filter::gaussian_blur_f32;
use tokio::time::{interval, Duration};
use once_cell::sync::Lazy;
use image::DynamicImage;
use serde_json::Value;
use reqwest::Client;

static KI: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
static TOKEN: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static LAST_SONG: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[tokio::main]
async fn main() {
    let home_dir = std::env::var("HOME").expect("Could not find the HOME environment variable");
    let output_dir = PathBuf::from(home_dir).join(".images");
    fs::create_dir_all(&output_dir).expect("");

    refresh_token().await;

    let refresh_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(35 * 60));
        loop {
            interval.tick().await;
            refresh_token().await;
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
                            Ok(x) => match x {
                                true => println!("song: {}", name),
                                false => {}
                            },
                            Err(_) => {
                                *LAST_SONG.lock().unwrap() = "".to_string();
                                println!("Error getting song image")
                            }
                        }
                    }
                }
            }
        } else if res.status().as_u16() == 429 {
            if *KI.lock().unwrap() >= KEYS.len() { *KI.lock().unwrap() = 0 }
            else { *KI.lock().unwrap() += 1 }
            *LAST_SONG.lock().unwrap() = "".to_string();
            println!("Changing keys: {}", *KI.lock().unwrap());
            refresh_token().await;
        }
    }
}

async fn show(img_url: &str, output_dir: &PathBuf, name: &str, screen_width: u32, screen_height: u32) -> Result<bool, Box<dyn std::error::Error>> {
    if &*LAST_SONG.lock().unwrap().as_str() == name { return Ok(false) }
    *LAST_SONG.lock().unwrap() = name.to_string();

    let output_path= output_dir.join(format!("{}.png", name));
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        if entry.path() == output_path {
            Command::new("wal")
                .args(&["-qeti", output_path.to_str().unwrap()])
                .output()?;
            return  Ok(true)
        }
    }

    let response = reqwest::get(img_url).await?;
    let bytes = response.bytes().await?;
    let original_image = image::load_from_memory(&bytes)?;

    let resized_image = original_image.resize_exact(screen_width, screen_height, image::imageops::FilterType::Lanczos3);

    let blurred_image = gaussian_blur_f32(&resized_image.to_rgba8(), 15.0);

    let mut canvas = DynamicImage::new_rgb8(screen_width, screen_height);
    image::imageops::overlay(&mut canvas, &blurred_image, 0, 0);
    image::imageops::overlay(
        &mut canvas,
        &original_image,
        (screen_width / 2 - original_image.width() / 2) as i64,
        (screen_height / 2 - original_image.height() / 2) as i64,
    );

    canvas.save(&output_path)?;

    Command::new("wal")
        .args(&["-qeti", output_path.to_str().unwrap()])
        .output()?;
    Ok(true)
}

async fn refresh_token() {
    let i = KI.lock().unwrap().to_owned();
    let credentials = general_purpose::STANDARD.encode(format!("{}:{}", KEYS[i].0, KEYS[i].1));
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", KEYS[i].2),
    ];

    let response = Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", credentials))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(access_token) = json["access_token"].as_str() {
                    *TOKEN.lock().unwrap() = access_token.to_string();
                    println!("Token refreshed.")
                }
            }
        }
        _ => {
            eprintln!("Token refresh failed. Retrying...");
            Box::pin(refresh_token()).await;
        }
    }
}