use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{PathBuf};
use anyhow::{anyhow, Result};
use humantime::Duration;
use std::time::SystemTime;
use log::{debug, error, info, warn};

// Create a new instance of Cache and read the cache from the file
pub fn read(cache_dir: PathBuf, ttl: humantime::Duration) -> Result<Cache> {
    let mut cache = Cache::new(cache_dir, ttl);
    debug!("Init cache: {:?}", &cache);
    info!("Reading cache from fle: {:?}", cache.file.as_path());
    cache.read()?;
    debug!("Read cache from {:?}", cache.data);
    Ok(cache)
}

#[derive(Debug)]
pub struct Cache {
    pub file: PathBuf,
    pub ttl: Duration,
    pub data: Data,
}

impl Cache {
    pub fn new(cache_dir: PathBuf, ttl: Duration) -> Cache {
        Cache {
            file: cache_dir.join("cache.json"),
            data: Data::NotChecked,
            ttl,
        }
    }

    pub fn write(&mut self, data: &str) -> Result<()> {
        let mut file = File::create(&self.file)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    // pub fn update(&mut self, data: &str) -> Result<()> {
    //     let mut file = File::open(&self.file)?;
    //     let mut data = String::new();
    //     file.read_to_string(&mut data)?;
    //     match serde_json::from_str::<Data>(&data) {
    //         Ok(data) => {
    //             self.data = data;
    //             Ok(())
    //         },
    //         Err(e) => {
    //             error!("Error parsing cache data: {}", e);
    //         }
    //     }
    // }

    pub fn read(&mut self) -> Result<&Self> {
        match File::open(&self.file) {
            Ok(mut f) => {
                info!("Cache file exists: {:?}", &self.file);
                info!("Reading cache file...");
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                debug!("Cache file read: {:?}", &s);

                info!("Checking cache file age...");
                let mills_since_modified = Duration::from(SystemTime::now().duration_since(f.metadata()?.modified()?)?);
                info!("Cache file age: {}", mills_since_modified);
                self.data = if mills_since_modified.as_micros() > self.ttl.as_micros() {
                    info!("Cache file is outdated");
                    Data::Outdated(s)
                } else {
                    info!("Cache file is not outdated");
                    Data::Ready(s)
                }
            }
            Err(e) => {
                warn!("Error opening cache file: {:?}", e);
                if e.kind() == ErrorKind::NotFound {
                    info!("Cache file does not exist: {:?}", &self.file);
                    info!("Creating cache file...");
                    File::create(&self.file)?;
                    self.data = Data::None
                } else {
                    return Err(anyhow!("Error opening cache file: {}", e));
                }
            }
        }
        Ok(self)
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Ready(String),
    Outdated(String),
    None,
    NotChecked,
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::str::FromStr;
    use tempfile;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    // Test Cache.get_cache() fail
    #[test]
    fn get_cache_fail() {
        let mut cache = Cache::new(PathBuf::from("/nonexist/"), Duration::from_str("1s").unwrap());
        assert!(cache.read().is_err());
    }

    // Test Cache.get_cache() not found
    #[test]
    fn get_cache_none() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir").into_path();
        let mut cache = Cache::new(dir.clone(), Duration::from_str("1s").unwrap());
        assert!(matches!(cache.read(), Ok(_)));
        assert!(matches!(cache.data, Data::None));
        // Check file is created
        assert!(
            !File::open(format!("{}/cache.json", dir.to_str().unwrap())).is_err()
        )
    }

    // Test Cache.get_cache() outdated
    #[test]
    fn get_cache_outdated() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir").into_path();
        let mut cache = Cache::new(dir.clone(), Duration::from_str("0 ns").unwrap());
        let mut file = File::create(dir.clone().join("cache.json")).unwrap();
        write!(file, "data").expect("Fila should contain info");
        assert!(matches!(cache.read(), Ok(_)));
        assert_eq!(cache.data, Data::Outdated("data".to_string()));
        // Check file is created
        assert!(
            !File::open(format!("{}/cache.json", dir.to_str().unwrap())).is_err()
        )
    }

    // Test Cache.get_cache() ready
    #[test]
    fn get_cache_ready() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir").into_path();
        let mut cache = Cache::new(dir.clone(), Duration::from_str("1 s").unwrap());
        let mut file = File::create(dir.clone().join("cache.json")).unwrap();
        write!(file, "data").expect("Fila should contain info");
        assert!(matches!(cache.read(), Ok(_)));
        assert_eq!(cache.data, Data::Ready("data".to_string()));
        // Check file is created
        assert!(
            !File::open(format!("{}/cache.json", dir.to_str().unwrap())).is_err()
        )
    }

    // Test Cache.write() fail
    #[test]
    fn write_fail() {
        let dir = PathBuf::from_str("/nonexist/").unwrap();
        let mut cache = Cache::new(dir.clone(), Duration::from_str("1s").unwrap());
        assert!(cache.write("data").is_err());
    }

    // Test Cache.write() success
    #[test]
    fn write_success() {
        let dir = tempfile::tempdir().expect("Can't create a temp dir").into_path();
        let mut cache = Cache::new(dir.clone(), Duration::from_str("1s").unwrap());
        assert!(matches!(cache.write("data"), Ok(_)));
        assert!(
            !File::open(format!("{}/cache.json", dir.to_str().unwrap())).is_err()
        )
    }
}