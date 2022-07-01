use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use colored::Colorize;

pub mod api;
pub mod cache;
pub mod config;


pub fn get_data(url: url::Url, mut c: cache::Cache) -> Result<api::Statistics> {
    info!("Trying to decide — get data from API or from cache");
    let data = c.data.clone();
    match data {
        cache::Data::Ready(ref s) => {
            info!("Cache is ready, no need to get API");
            serde_json::from_str(s).context("Error parsing cache data")
        },
        cache::Data::Outdated(ref s) => {
            info!("Cache is outdated");
            api::get_data(url)
                .and_then(|d| serde_json::from_str(&d).context("Error parsing API data"))
                .and_then(|d: api::Statistics| {
                    info!("Got data from API, writing cache");
                    info!("Cache data: {:?}", &d);
                    c.write(s)?;
                    info!("Cache written");
                    Ok(d)
                })
                .or_else(|_| serde_json::from_str(s).context("Error parsing cache data"))
        },
        cache::Data::None => {
            info!("Cache is empty");
            info!("Getting data from API");
            let data = api::get_data(url)?;
            debug!("Got data from API: {:?}", &data);
            info!("Got data from API, trying to parse it");
            let result: api::Statistics = serde_json::from_str(&data).context("Error serializing data")?;
            debug!("Parsed data: {:?}", &result);
            info!("Parsed data, writing cache");
            c.write(&data)?;
            info!("Cache written");
            Ok(result)
        }
        _ => Err(anyhow!("Cache in wired state...")),
    }
}

pub fn print_result(data: &api::Statistics) {
    let mil = data.data.stats.personnel_units.to_string().red();
    let mil_inc = data.data.increase.personnel_units.to_string().green();
    print!("{}↑{}", mil, mil_inc)
}
