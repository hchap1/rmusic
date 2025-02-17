use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::process::Command;
use std::process::Stdio;
use thirtyfour::prelude::*;

use crate::downloader::Song;

pub async fn search_youtube(query: String) -> Result<Vec<Song>, String> {
    let mut chromedriver = match Command::new("chromedriver").stdout(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(e) => return Err(format!("Failed to spawn chromedriver: {e:?}"))
    };

    let stdout = match chromedriver.stdout.take() {
        Some(stdout) => stdout,
        None => return Err(String::from("Failed to capture STDOUT of chromedriver."))
    };

    let mut reader = BufReader::new(stdout).lines();

    for _ in 0..3 { let _ = reader.next_line().await; }

    let ip = match reader.next_line().await {
        Ok(line) => line.unwrap().split(" ").nth(6).unwrap().to_string().strip_suffix('.').unwrap().to_string(),
        Err(e) => return Err(format!("Failed to read STDOUT of chromedriver: {e:?}"))
    };

    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_arg("--headless");
    let _ = caps.add_arg("--disable-gpu");
    let _ = caps.add_arg("--no-sandbox");

    let driver = WebDriver::new(format!("http://localhost:{ip}"), caps).await.unwrap();

    driver.goto(format!("https://youtube.com/results?search_query={}", query)).await.unwrap();
    let video_titles = driver.find_all(By::Id("video-title")).await.unwrap();
    let video_channels = driver.find_all(By::Id("channel-name")).await.unwrap();

    let mut options: Vec<Song> = Vec::new();

    for idx in 0..video_titles.len() {
        let title = video_titles[idx].text().await.unwrap();
        let url_idx = video_titles[idx].outer_html().await.unwrap().find("href").unwrap();
        let url_slice = &video_titles[idx].outer_html().await.unwrap().to_string()[url_idx..];
        let url_end = url_slice.find(">").unwrap() - 1 + url_idx;
        let url_slice = String::from("https://youtube.com/") + &video_titles[idx].outer_html().await.unwrap().to_string()[url_idx + 7..url_end];
        let channel = video_channels[idx * 2 + 1].text().await.unwrap();

        options.push(Song {
            name: title,
            channel,
            url: url_slice,
            file: None
        });
    }

    driver.quit().await.unwrap();
    Ok(options)
}
