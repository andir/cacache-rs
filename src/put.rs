//! Functions for writing to cache.
use std::path::{Path, PathBuf};

use nix::unistd::{Uid, Gid};
use serde_json::Value;
use ssri::Integrity;

use crate::content::write;
use crate::index;
use crate::errors::Error;

pub fn data<P: AsRef<Path>, D: AsRef<[u8]>, K: AsRef<str>>(cache: P, key: K, data: D) -> Result<Integrity, Error> {
    let sri = write::write(cache.as_ref(), data.as_ref())?;
    Writer::new(cache.as_ref(), key.as_ref()).integrity(sri).commit(data)
}

pub struct Writer {
    pub cache: PathBuf,
    pub key: String,
    pub sri: Option<Integrity>,
    pub size: Option<usize>,
    pub time: Option<u128>,
    pub metadata: Option<Value>,
    pub uid: Option<Uid>,
    pub gid: Option<Gid>,
}

impl Writer {
    pub fn new<P: AsRef<Path>, K: AsRef<str>>(cache: P, key: K) -> Writer {
        Writer {
            cache: cache.as_ref().to_path_buf(),
            key: String::from(key.as_ref()),
            sri: None,
            size: None,
            time: None,
            metadata: None,
            uid: None,
            gid: None
        }
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn time(mut self, time: u128) -> Self {
        self.time = Some(time);
        self
    }

    pub fn integrity(mut self, sri: Integrity) -> Self {
        self.sri = Some(sri);
        self
    }

    pub fn chown(mut self, uid: Option<Uid>, gid: Option<Gid>) -> Self {
        self.uid = uid;
        self.gid = gid;
        self
    }

    pub fn commit<D: AsRef<[u8]>>(self, data: D) -> Result<Integrity, Error> {
        if let Some(sri) = &self.sri {
            if sri.clone().check(&data).is_none() {
                return Err(Error::IntegrityError);
            }
        }
        if let Some(size) = self.size {
            if size != data.as_ref().len() {
                return Err(Error::SizeError);
            }
        }
        let sri = write::write(&self.cache, data.as_ref())?;
        index::insert(self)?;
        Ok(sri)
    }
}