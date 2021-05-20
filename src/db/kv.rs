// Copyright 2021 Anthony Mugendi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Copyright 2021 Anthony Mugendi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use kv;

use serde::*;
use sled::IVec;
use wildmatch::WildMatch;

use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
    result::Result,
    str,
};

use once_cell::sync::Lazy;
use std::sync::Mutex;
static KV_PATH: Lazy<Mutex<HashMap<&str, PathBuf>>> = Lazy::new(|| {
    //Index Mutex
    let m: HashMap<&str, PathBuf> = HashMap::new();
    Mutex::new(m)
});

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Types {
    String(String),

    // bool
    Bool(bool),

    // numbers
    I16(i16),
    I32(i32),
    I64(i64),
    U16(u16),
    U32(u32),
    U64(u64),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Record {
    pub name: String,
    pub values: Vec<Types>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KVRecord {
    pub key: String,
    pub record: Record,
}

#[derive(Debug, Deserialize)]
enum KVMethods<'a> {
    Get(&'a str),
    Remove(&'a str),
    Set(&'a str, Record),
    Transaction(HashMap<String, Record>),
}

#[derive(Debug)]
pub struct Filter<'f> {
    pub key: Option<&'f str>,
    pub name: Option<&'f str>,
}


pub fn init(path_opt: Option<&Path>) -> Result<(), Box<dyn Error>> {
    // get mutex value
    let mut path = KV_PATH.lock().unwrap();

    match path_opt {
        Some(db_path) => {
            // ensure path exists
            if !db_path.exists() {
                println!("Creating directory path {:?} ", db_path);
                create_dir_all(db_path)?;
            }
            path.insert("kv_path", db_path.to_path_buf());
        }
        _ => {}
    }

    Ok(())
}

pub fn reset(path_opt: Option<&Path>) -> Result<(), Box<dyn Error>> {
    match path_opt {
        Some(db_path) => {
            // ensure path exists
            if !db_path.exists() {
                panic!("Path provided {:?} does not exist!", db_path);
            }

            // path.insert("kv_path", db_path.to_path_buf());
            let _ = remove_dir_all(&db_path)?;
        }
        _ => {}
    }

    Ok(())
}

fn get_store() -> Result<kv::Store, Box<dyn Error>> {
    init(None)?;

    let path_mutex = KV_PATH.lock().unwrap();
    let index_path = &path_mutex["kv_path"];

    // Configure the database
    let cfg = kv::Config::new(index_path);

    // Open the key/value store
    let store = kv::Store::new(cfg)?;

    // A Bucket provides typed access to a section of the key/value store
    // let this_bucket = store.bucket::<kv::Raw, kv::Raw>(Some(bucket))?;

    Ok(store)
}

fn exec(_bucket: &str, method: KVMethods) -> Result<Option<KVRecord>, Box<dyn Error>> {
    let store = get_store()?;
    let bucket = store.bucket::<kv::Raw, kv::Bincode<Record>>(Some(_bucket))?;

    match method {
        KVMethods::Set(key, val) => {
            bucket.set(key.as_bytes(), kv::Bincode(val))?;
            Ok(None)
        }
        KVMethods::Transaction(ts) => {
            // start transaction
            bucket.transaction(|txn| {
                // add all values
                for (key, val) in &ts {
                    txn.set(key.as_bytes(), kv::Bincode(val.clone()))?;
                }
                // finish up
                Ok(())
            })?;

            // bucket.set(key.as_bytes(), kv::Bincode(val))?;
            Ok(None)
        }
        KVMethods::Get(key) => {
            //  get value
            let value = bucket.get(key.as_bytes()).unwrap();

            if value.is_some() {
                let bincode = value.unwrap();
                let result = KVRecord {
                    key: key.into(),
                    record: bincode.0,
                };
                Ok(Some(result))
            } else {
                Ok(None)
            }
        }
        KVMethods::Remove(key) => {
            //  get value
            bucket.remove(key.as_bytes()).unwrap();
            Ok(None)
        }
    }
}

pub fn get(bucket: &str, key: &str) -> Option<KVRecord> {
    exec(bucket, KVMethods::Get(key)).expect("Could not get key")
}

pub fn remove(bucket: &str, key: &str) -> Result<(), Box<dyn Error>> {
    exec(bucket, KVMethods::Remove(key))?;
    Ok(())
}

pub fn set(bucket: &str, key: &str, val: Record) -> Result<(), Box<dyn Error>> {
    // https://docs.rs/kv/0.22.0/kv/
    exec(bucket, KVMethods::Set(key, val))?;

    Ok(())
}

pub fn transaction(bucket: &str, ts: HashMap<String, Record>) -> Result<(), Box<dyn Error>> {
    exec(bucket, KVMethods::Transaction(ts))?;

    Ok(())
}

pub fn iter(bucket: &str) -> kv::Iter<kv::Raw, kv::Bincode<Record>> {
    let store = get_store().expect("Could not get store");
    let bucket = store
        .bucket::<kv::Raw, kv::Bincode<Record>>(Some(bucket))
        .expect("Could not get bucket");

    bucket.iter()
}

pub fn get_all(bucket: &str, filter: Option<Filter>) -> Vec<KVRecord> {
    let iter = iter(bucket);

    let mut iter_resp: Vec<KVRecord> = Vec::new();

    iter.enumerate().for_each(|(_, item)| {
        let item = item.unwrap();

        let bincode = item.value::<kv::Bincode<Record>>().unwrap();
        let record: Record = bincode.0;
        let key_ivec: IVec = item.key::<IVec>().unwrap();
        let key = str::from_utf8(&key_ivec).expect("Could not convert ivec to string");

        let mut filtered_ok: bool = true;

        match &filter {
            Some(f) => {
                // filter by key first
                if f.key.is_some() {
                    let _key = f.key.as_ref().unwrap();
                    filtered_ok = WildMatch::new(_key).matches(key);
                }

                //  filter by name if filtered_ok is still ok
                if f.name.is_some() && filtered_ok {
                    let _name = f.name.as_ref().unwrap();
                    filtered_ok = WildMatch::new(_name).matches(&record.name);
                }
            }
            None => {}
        }

        if filtered_ok {
            iter_resp.push(KVRecord {
                key: key.into(),
                record,
            });
        }
    });

    iter_resp
}
