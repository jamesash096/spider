use std::collections::HashSet;
use std::time::Instant;
use rayon::prelude::*;

fn main() -> spider::Result<()>{
    let now = Instant::now();

    let client = reqwest::blocking::Client::new();
    let origin_url = "https://www.ets.org/";
    
    let body = spider::fetch_url(&client, origin_url)?;

    spider::write_file("", &body)?;

    let mut visited = HashSet::new();
    visited.insert(origin_url.to_string());
    let found_urls = spider::get_links_from_site(&body);
    let mut new_urls = found_urls.difference(&visited).map(|x| x.to_string()).collect::<HashSet<String>>();

    while !new_urls.is_empty() {
        let (found_urls, errors): (Vec<spider::Result<HashSet<String>>>, Vec<_>) = new_urls.par_iter().map(|url| -> spider::Result<HashSet<String>>{
            let body = spider::fetch_url(&client, url)?;
            spider::write_file(&url[origin_url.len() - 1..], &body)?;
            
            let links = spider::get_links_from_site(&body);
            println!("Visited: {} found {} links", url, links.len());
            Ok(links)
        }).partition(Result::is_ok);
        visited.extend(new_urls);

        new_urls = found_urls.into_par_iter().map(Result::unwrap).reduce(HashSet::new, |mut acc,x|{
            acc.extend(x);
            acc
        }).difference(&visited).map(|x| x.to_string()).collect::<HashSet<String>>();
        println!("New urls: {}",new_urls.len());
        println!("Errors: {:#?}", errors.into_iter().map(Result::unwrap_err).collect::<Vec<spider::Error>>())
    }

    //println!("URLS: {:#?}", found_urls);
    println!("{}", now.elapsed().as_secs());
    Ok(())
}
