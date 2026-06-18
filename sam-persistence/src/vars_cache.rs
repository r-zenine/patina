use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTimeError;
use thiserror::Error;

use crate::associative_state::AssociativeStateWithTTL;
use crate::associative_state::ErrorAssociativeState;

pub trait VarsCache {
    fn put(
        &self,
        name: &dyn AsRef<str>,
        command: &dyn AsRef<str>,
        output: &dyn AsRef<str>,
        duration: Duration,
    ) -> Result<(), CacheError>;
    fn get(&self, command: &dyn AsRef<str>) -> Result<Option<String>, CacheError>;
}

#[derive(Debug)]
pub struct RustBreakCache {
    state: AssociativeStateWithTTL<CacheEntry>,
    min_cache_duration: Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct CacheEntry {
    pub name: String,
    pub command: String,
    pub output: String,
}

impl RustBreakCache {
    pub fn with_ttl(
        p: impl AsRef<Path>,
        ttl: &Duration,
        min_cache_duration: Duration,
    ) -> Result<Self, CacheError> {
        Ok(RustBreakCache {
            state: AssociativeStateWithTTL::<CacheEntry>::with_ttl(p, ttl)?,
            min_cache_duration,
        })
    }

    pub fn entries(&self) -> Result<impl Iterator<Item = CacheEntry>, CacheError> {
        Ok(self.state.entries()?.map(|(_, v)| v))
    }

    pub fn delete(&self, key: &str) -> Result<Option<CacheEntry>, CacheError> {
        Ok(self.state.delete(key)?)
    }

    pub fn clear_cache(&self) -> Result<(), CacheError> {
        for (key, _) in self.state.entries()? {
            self.state.delete(key)?;
        }
        Ok(())
    }
}

impl VarsCache for RustBreakCache {
    fn put(
        &self,
        name: &dyn AsRef<str>,
        command: &dyn AsRef<str>,
        output: &dyn AsRef<str>,
        duration: Duration,
    ) -> Result<(), CacheError> {
        if duration < self.min_cache_duration {
            return Ok(());
        }
        let key = command.as_ref().to_string();
        let entry = CacheEntry {
            name: name.as_ref().to_string(),
            command: key.clone(),
            output: output.as_ref().to_string(),
        };
        Ok(self.state.put(key, entry)?)
    }

    fn get(&self, command: &dyn AsRef<str>) -> Result<Option<String>, CacheError> {
        let cache_key = command.as_ref();
        Ok(self.state.get(cache_key)?.map(|v| v.output))
    }
}

pub struct NoopVarsCache {}

impl VarsCache for NoopVarsCache {
    fn put(
        &self,
        _name: &dyn AsRef<str>,
        _command: &dyn AsRef<str>,
        _output: &dyn AsRef<str>,
        _duration: Duration,
    ) -> Result<(), CacheError> {
        Ok(())
    }
    fn get(&self, _command: &dyn AsRef<str>) -> Result<Option<String>, CacheError> {
        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("can't interract with rustbreak because\n-> {0}")]
    RustbreakError(#[from] rustbreak::RustbreakError),
    #[error("could not get a timestamp from the system because\n-> {0}")]
    CantGetTimeStamp(#[from] SystemTimeError),
    #[error("could not interract with cache because\n-> {0}")]
    ErrAssociativeState(#[from] ErrorAssociativeState),
}

#[cfg(test)]
mod tests {
    use crate::vars_cache::{RustBreakCache, VarsCache};
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    pub fn test_rustbreak_cache() {
        let tmp_dir = NamedTempFile::new().expect("can't create a temporary file");
        let ttl = Duration::from_secs(90);
        let min_duration = Duration::ZERO;
        let cache =
            RustBreakCache::with_ttl(tmp_dir.path(), &ttl, min_duration).expect("Can't open cache");
        cache
            .put(
                &String::from("name"),
                &String::from("command"),
                &String::from("output"),
                Duration::from_secs(5),
            )
            .expect("can't write in rustbreak cache");

        let cache2 =
            RustBreakCache::with_ttl(tmp_dir.path(), &ttl, min_duration).expect("Can't open cache");
        let value = cache2
            .get(&String::from("command"))
            .expect("can't read from rustbreak cache")
            .expect("can't retrieve the value from rustbreak cache");
        assert_eq!(value, "output");

        let cache =
            RustBreakCache::with_ttl(tmp_dir.path(), &ttl, min_duration).expect("Can't open cache");
        cache
            .put(
                &String::from("name"),
                &String::from("command2"),
                &String::from("output"),
                Duration::from_secs(5),
            )
            .expect("can't write in rustbreak cache");

        let value = cache2
            .get(&String::from("command2"))
            .expect("can't read from rustbreak cache")
            .expect("can't retrieve the value from rustbreak cache");
        assert_eq!(value, "output");
    }

    #[test]
    pub fn test_rustbreak_cache_admission_threshold() {
        let tmp_dir = NamedTempFile::new().expect("can't create a temporary file");
        let ttl = Duration::from_secs(90);
        let min_duration = Duration::from_secs(2);
        let cache =
            RustBreakCache::with_ttl(tmp_dir.path(), &ttl, min_duration).expect("Can't open cache");

        // Fast command: below threshold, should not be stored
        cache
            .put(
                &String::from("name"),
                &String::from("fast_cmd"),
                &String::from("output"),
                Duration::from_millis(500),
            )
            .expect("can't write in rustbreak cache");
        let value = cache
            .get(&String::from("fast_cmd"))
            .expect("can't read from rustbreak cache");
        assert!(value.is_none(), "fast command should not be cached");

        // Slow command: at or above threshold, should be stored
        cache
            .put(
                &String::from("name"),
                &String::from("slow_cmd"),
                &String::from("output"),
                Duration::from_secs(3),
            )
            .expect("can't write in rustbreak cache");
        let value = cache
            .get(&String::from("slow_cmd"))
            .expect("can't read from rustbreak cache");
        assert!(value.is_some(), "slow command should be cached");
    }
}
