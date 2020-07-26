use thirtyfour::{prelude::*, ChromeCapabilities};
use tokio;
use futures::future::join_all;
use std::{
    io::{self, BufRead},
    fs::File,
};

#[derive(Debug)]
struct Site {
    url: String,
    publisher: String,
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let serveraddr = "http://localhost:9515";

    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;

    let file = io::BufReader::new(File::open("sites.csv").expect("Can't open file"));
    let lines = file.lines();

    let sites: Vec<Site> = lines.map(|line| {
        let line = line.expect("Can't read line");
        let parts: Vec<&str> = line.split(',').collect();
        Site {
            url: parts[0].to_string(),
            publisher: parts[2].to_string(),
        }
    }).collect();

    // let links = [
    //     "https://www.youtube.com/watch?v=HJigew8ZAko",
    //     "https://www.youtube.com/watch?v=NzIBVnNL-ko",
    //     "https://www.youtube.com/watch?v=iJqUGBxi8O4",
    // ];


    
    let fut = join_all(sites
        .into_iter()
        .map(|site| get_youtube_viewcount(site, serveraddr, &caps)));

    let results = fut.await;

    for result in results {
        let unwraped = result?;
        println!("{:#?} counts at {:#?}", unwraped.0, unwraped.1);
    }
    Ok(())
}

async fn get_youtube_viewcount(
    site: Site,
    serveraddr: &str,
    caps: &ChromeCapabilities,
) -> WebDriverResult<(u32, Site)> {
    let driver = WebDriver::new(serveraddr, caps).await?;

    driver.get(site.url.clone()).await?;
    let viewspans = driver.find_elements(By::ClassName("view-count")).await?;
    for viewspan in &viewspans {
        println!("views: {}", viewspan.text().await?);
    }

    let element = viewspans.first().expect("Can't get view element");
    let words = element.text().await?;
    let words: Vec<&str> = words.split(" ").collect();
    let count: u32 = words
        .first()
        .expect("Can't get view count")
        .parse()
        .expect("Can't convert view count");
    Ok((count, site))
}
