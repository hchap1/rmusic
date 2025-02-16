mod chromedriver;

use chromedriver::headless_browser;

#[tokio::main]
async fn main() {

    let _ = headless_browser(String::from("viva la vida")).await;

}
