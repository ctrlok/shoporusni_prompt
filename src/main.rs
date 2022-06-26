use std::fs::{File, DirBuilder};
use anyhow::{Context, Result, Error};
use std::io::{ErrorKind, Read, Write};
use std::time::{Duration, SystemTime};
use clap::Parser;
use log::{info, warn, debug};
use simplelog::ColorChoice;
use reqwest;
use humantime;
use serde_derive::Deserialize;
use serde_derive;
use colored::Colorize;
use colored::control;

use serde_json;

#[derive(Debug, Parser)]
struct Cli {
    /// -v for warn, -vv for info, -vvv for debug
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Remote API URL
    #[clap(short, long, value_parser, default_value = "https://russianwarship.rip/api/v1/statistics/latest")]
    url: url::Url,

    /// Refresh time for cache
    #[clap(short, long, value_parser, default_value = "30minutes")]
    refresh: humantime::Duration,

}


enum CacheResponse {
    Outdated(String),
    Exist(String),
    None,
}

fn main() -> Result<()> {
    control::set_override(true);
    let c: Cli = Cli::parse();
    if atty::is(atty::Stream::Stdout) {
        simplelog::TermLogger::init(c.verbose.log_level_filter(), Default::default(), Default::default(), ColorChoice::Always).expect("Loggin init error");
    }
    let path = format!("{}/shoporusni/", dirs::config_dir().unwrap().to_str().unwrap());
    info!("Checking if config dir exist");
    DirBuilder::new()
        .recursive(true).create(&path).context("Creating a new config directory")?;
    let data = get_data(&c, &path)?;
    Ok(print_result(data))
}

fn print_result(data: Root) {
    let mil = format!("{}", data.data.stats.personnel_units).as_str().red();
    let mil_inc = format!("{}", data.data.increase.personnel_units).as_str().green();
    print!("{}↑{}", mil, mil_inc)
}


fn get_data(cli: &Cli, cache_path: &str) -> Result<Root> {
    info!("Starting to read the data");
    let data_string = match get_cache(cache_path, cli.refresh.into()).context("Reading data from cache")? {
        CacheResponse::Exist(data_from_cache) => {
            info!("Found data in cache, sending it!");
            data_from_cache
        }
        CacheResponse::Outdated(data_from_cache) => {
            info!("Data in cache is outdated, trying to get newest from API");
            get_data_from_api(cli).or_else::<String, _>(|e| {
                warn!("Cant get data from API with error {}", e);
                warn!("return data back from cache");
                Ok(data_from_cache)
            }).unwrap()
        }
        CacheResponse::None => {
            info!("Looks like we didn't have data in cache. Will get new one from API");
            get_data_from_api(cli)?
        }
    };

    debug!("We got our data: {}", data_string);
    info!("Starting data parsing!");
    let data: Root = serde_json::from_str(&*data_string)?;
    debug!("Data parsed as {:?}", data);
    update_cache(&data_string, cache_path)?;
    debug!("{:?}", &data.data);
    Ok(data)
}


fn update_cache(data: &str, path: &str) -> Result<()> {
    let mut f = File::options().write(true).open(format!("{}/cache.json", path))?;
    f.write_all(data.as_bytes())?;
    Ok(())
}

fn get_data_from_api(cli: &Cli) -> Result<String> {
    info!("Getting data from API: {}", cli.url.as_str());
    let s = reqwest::blocking::get(
        cli.url.as_str()
    )?.text()?;
    Ok(s)
}

fn get_cache(dir: &str, timeout: Duration) -> Result<CacheResponse> {
    info!("Trying to reach out data from cache");
    let cachefile_path = format!("{}/cache.json", dir);
    match File::open(&cachefile_path) {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            if SystemTime::now().duration_since(f.metadata()?.modified()?)? > timeout {
                info!("Cache is a little bit outdated");
                Ok(CacheResponse::Outdated(s))
            } else {
                info!("Cache is good as new!");
                Ok(CacheResponse::Exist(s))
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                warn!("Cache file not found. Trying to create a new one");
                File::create(&cachefile_path)
                    .context(format!("Creating file: {}", &cachefile_path))?;
                Ok(CacheResponse::None)
            } else {
                Err(Error::from(e))
            }
        }
    }
}


#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub message: String,
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub date: String,
    pub day: i64,
    pub resource: String,
    pub stats: Stats,
    pub increase: Increase,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    #[serde(rename = "personnel_units")]
    pub personnel_units: i64,
    pub tanks: i64,
    #[serde(rename = "armoured_fighting_vehicles")]
    pub armoured_fighting_vehicles: i64,
    #[serde(rename = "artillery_systems")]
    pub artillery_systems: i64,
    pub mlrs: i64,
    #[serde(rename = "aa_warfare_systems")]
    pub aa_warfare_systems: i64,
    pub planes: i64,
    pub helicopters: i64,
    #[serde(rename = "vehicles_fuel_tanks")]
    pub vehicles_fuel_tanks: i64,
    #[serde(rename = "warships_cutters")]
    pub warships_cutters: i64,
    #[serde(rename = "cruise_missiles")]
    pub cruise_missiles: i64,
    #[serde(rename = "uav_systems")]
    pub uav_systems: i64,
    #[serde(rename = "special_military_equip")]
    pub special_military_equip: i64,
    #[serde(rename = "atgm_srbm_systems")]
    pub atgm_srbm_systems: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Increase {
    #[serde(rename = "personnel_units")]
    pub personnel_units: i64,
    pub tanks: i64,
    #[serde(rename = "armoured_fighting_vehicles")]
    pub armoured_fighting_vehicles: i64,
    #[serde(rename = "artillery_systems")]
    pub artillery_systems: i64,
    pub mlrs: i64,
    #[serde(rename = "aa_warfare_systems")]
    pub aa_warfare_systems: i64,
    pub planes: i64,
    pub helicopters: i64,
    #[serde(rename = "vehicles_fuel_tanks")]
    pub vehicles_fuel_tanks: i64,
    #[serde(rename = "warships_cutters")]
    pub warships_cutters: i64,
    #[serde(rename = "cruise_missiles")]
    pub cruise_missiles: i64,
    #[serde(rename = "uav_systems")]
    pub uav_systems: i64,
    #[serde(rename = "special_military_equip")]
    pub special_military_equip: i64,
    #[serde(rename = "atgm_srbm_systems")]
    pub atgm_srbm_systems: i64,
}


#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn get_cache_fail() {
        assert!(get_cache(&String::from("/nonexist/"), Default::default()).is_err())
    }

    #[test]
    fn get_cache_none() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir");
        assert!(matches!(
            get_cache(dir.path().to_str().unwrap(), Default::default()), Ok(CacheResponse::None)
        ));
        // Check file is created
        assert!(
            !File::open(format!("{}/cache.json", dir.path().to_str().unwrap())).is_err()
        )
    }

    #[test]
    fn get_cache_outdated() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir");
        let mut file = File::create(format!("{}/cache.json", dir.path().to_str().unwrap())).unwrap();
        write!(file, "data").expect("Fila should contain info");
        assert!(matches!(
            get_cache(dir.path().to_str().unwrap(), Duration::new(0,1)).unwrap(), CacheResponse::Outdated(_)
        ))
    }

    #[test]
    fn get_cache_succ() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir");
        let mut file = File::create(format!("{}/cache.json", dir.path().to_str().unwrap())).unwrap();
        write!(file, "data").expect("Fila should contain info");
        assert!(matches!(
            get_cache(dir.path().to_str().unwrap(), Duration::new(1,0)).unwrap(), CacheResponse::Exist(_)
        ))
    }
}

