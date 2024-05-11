use std::{str::FromStr, thread, time::Duration};

use anyhow::anyhow;

use thirtyfour::{By, DesiredCapabilities, WebDriver};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Debug)]
enum Page {
    Evapolak,
    Artmajeur,
}

impl Page {
    async fn process_url(&self, driver: &WebDriver) -> anyhow::Result<String> {
        thread::sleep(Duration::from_secs_f32(0.2));

        match self {
            Page::Artmajeur => {
                let elem = driver
                    .find(By::ClassName("d-sm-flex"))
                    .await?
                    .find(By::ClassName("order-sm-1"))
                    .await?;

                let img_title = elem
                    .find(By::ClassName("text-uppercase"))
                    .await?
                    .text()
                    .await?;

                let ph_name = elem.find(By::Tag("a")).await?.text().await?;
                let img_url = driver
                    .find(By::ClassName("img-main"))
                    .await?
                    .prop("src")
                    .await?;

                if let Some(url_string) = img_url {
                    let mut clean_title = img_title.replace('"', "");
                    clean_title = format!("images/{}-{}.jpg", clean_title, ph_name);
                    match download_image(url_string.as_str(), clean_title.as_str()).await {
                        Ok(_) => Ok(clean_title.to_owned()),
                        Err(err) => Err(err),
                    }
                } else {
                    Err(anyhow!("Image URL not found!"))
                }
            }
            Page::Evapolak => {
                let elem = driver
                    .find(By::ClassName("art-print-products-show"))
                    .await?;

                let img_title = elem.find(By::Tag("h1")).await?.text().await?;

                let img_url = driver
                    .find(By::ClassName("fancybox-button"))
                    .await?
                    .prop("href")
                    .await?;

                if let Some(url_string) = img_url {
                    let mut clean_title = img_title.replace('"', "");
                    clean_title = format!("images/{}-EvaPolak.jpg", clean_title);
                    match download_image(url_string.as_str(), clean_title.as_str()).await {
                        Ok(_) => Ok(clean_title.to_owned()),
                        Err(err) => Err(err),
                    }
                } else {
                    Err(anyhow!("Image URL not found!"))
                }
            }
        }
    }
}

impl FromStr for Page {
    type Err = color_eyre::Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.contains("artmajeur") {
            Ok(Page::Artmajeur)
        } else if value.contains("evapolak") {
            Ok(Page::Evapolak)
        } else {
            Err(color_eyre::eyre::eyre!("not a valid page name"))
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    let contents = fs::read_to_string("links.txt").await?;

    let urls: Vec<&str> = contents.lines().collect();

    for url in urls {
        driver.goto(url).await?;
        let page = Page::from_str(url).unwrap();
        match page.process_url(&driver).await {
            Ok(file_name) => println!("Saving into {}...", file_name),
            Err(err) => println!("error while downloading image: {}", err),
        }
    }

    Ok(())
}

async fn download_image(url: &str, file_name: &str) -> anyhow::Result<()> {
    let response = reqwest::get(url).await?;

    let mut file = File::create(file_name).await?;

    file.write_all(&response.bytes().await?).await?;

    Ok(())
}
