use std::fs;
use std::io::Read;
use std::io::Error as IoErr;
use std::path::Path;
use std::collections::HashSet;
use select::document::Document;
use select::predicate::Name;
use select::predicate::Predicate;
use reqwest::Url;

#[derive(Debug)]
pub enum Error{
    Write{ url: String, e: IoErr },
    Fetch{ url: String, e: reqwest::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

impl<S: AsRef<str>> From <(S,IoErr)> for Error {
    fn from((url,e): (S,IoErr)) -> Self {
        Error::Write {
            url: url.as_ref().to_string(),
            e,
        }
    }
}

impl<S: AsRef<str>> From <(S, reqwest::Error)> for Error {
    fn from((url,e): (S, reqwest::Error)) -> Error {
        Error::Fetch {
            url: url.as_ref().to_string(),
            e,
        }
    }
}

pub fn has_extension(url: &&str) -> bool {
    Path::new(url).extension().is_none()
}

pub fn normalize_url(url: &str) -> Option<String> {
    let new_url = Url::parse(url);
    match new_url {
        Ok(new_url) => {
            if let Some("ets.org") = new_url.host_str() {
                Some(url.to_string())
            } else {
                None
            }
        },
        Err(_e) => {
            if url.starts_with("/") {
                Some(format!("https://www.ets.org{}", url))
            } else {
                None
            }
        }
    }
}

pub fn get_links_from_site(html: &str) -> HashSet<String> {
    Document::from(html)
        .find(Name("a").or(Name("link")))
        .filter_map(|n| n.attr("href"))
        .filter(has_extension)
        .filter_map(normalize_url)
        .collect::<HashSet<String>>()
}

pub fn fetch_url(client: &reqwest::blocking::Client, url: &str) -> Result<String> {
    let mut res = client.get(url).send().map_err(|e| (url,e))?;
    println!("Status for {}: {}", url, res.status());

    let mut body = String::new();
    res.read_to_string(&mut body).map_err(|e| (url,e))?;
    Ok(body)
}

pub fn write_file(path: &str, content: &str) -> Result<()>{
    let dir = format!("static{}", path);
    fs::create_dir_all(format!("static{}", path)).map_err(|e| (&dir,e))?;
    let index = format!("static{}/index.html", path);
    fs::write(&index, content).map_err(|e| (&index,e))?;

    Ok(())
}