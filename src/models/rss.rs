use std::{thread::sleep, time::Duration};

//use log::{error, info, warn};
use quick_xml::de::from_str;
use reqwest::blocking::{self, Response};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
pub struct Item {
    pub title: String,
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
    pub link: String,
    pub author: String,
}

#[derive(Deserialize)]
pub struct Channel {
    pub title: String,
    pub description: String,
    #[serde(rename = "lastBuildDate", default)]
    pub last_build_date: String,
    pub item: Vec<Item>,
}

#[derive(Deserialize)]
pub struct Rss {
    pub channel: Channel,
}

fn get(url: &str, mut retry: i8) -> Result<Response, Box<dyn Error>> {
    match reqwest::blocking::get(url) {
        Ok(response) => Ok(response),
        Err(e) => {
            if retry > 0 {
                let interval = 15;
                std::thread::sleep(Duration::from_secs(interval));
                retry -= 1;
                get(url, retry)
            } else {
                Err(Box::new(e))
            }
        }
    }
}

impl Rss {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let retry: i8 = 5;
        let res = get(url, retry)?;
        let body = res.text()?;

        from_str(&body).map_err(|e| e.into())
    }
}
