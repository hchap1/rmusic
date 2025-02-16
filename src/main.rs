use std::{thread::sleep, time::Duration};

fn main() {

    let mut driver = WebDriver::new(Browser::Firefox);
    let url = String::from("https://youtube.com/results?search_query={}");

    driver.start_session().unwrap();
    driver.navigate(&url).unwrap();

    sleep(Duration::from_secs(2));

    let elements = driver.find_elements_by_css

}
