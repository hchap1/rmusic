use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::process::Command;
use std::time::Duration;
use std::process::Stdio;
use thirtyfour::prelude::*;

pub async fn headless_browser(query: String) -> Result<(), String> {
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
    let _ = tokio::time::sleep(Duration::from_secs(5)).await;
    let elem_form = driver.find_all(By::Id("video-title")).await.unwrap();
    for elem in elem_form { println!("{}", elem.text().await.unwrap()); }

    driver.quit().await.unwrap();

    
    Ok(())
}
