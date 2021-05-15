<!--
 Copyright 2021 Anthony Mugendi
 
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 
     http://www.apache.org/licenses/LICENSE-2.0
 
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->

# Extending the Basic KV Store
Key/Value stores are super fast and effective. and Sled is one of the best out there.

This crate seeks to better organize the basic K/V data so that it has much more meaning. Something closer to the kind of organization you would get in a document store but running on top of Sled for maximum speed.

The following example attempts to organize **Japanese Machines** into a dataset of vehicles & electronics.


```rust
pub mod db;

use blazeup::kv;

use std::{error::Error, path::Path, result::Result  };


fn main() -> Result<(), Box<dyn Error>> {
    // path where your database is to be stored
    // Note the folder is created if missing
    let path = Path::new("./db");

    // reset
    // Note reset deletes the entire KV directory
    kv::reset(Some(&path))?;

    //init with index path...
    kv::init(Some(&path))?;

    // set bucket name. 
    let bucket = "Japanese-Machines";

    
    //create your value
    //the value includes a name as well as a vector of the kv::Types enum values
    let vehicles = kv::Record{
        name:"models".into(), //should be a string
        values: vec![
            kv::Types::String("Toyota".into()), 
            kv::Types::String("Subaru".into()), 
        ]
    };

    
    // set value by passing key & value
    kv::set(&bucket, "vehicles", vehicles.clone())?;

    let tvs = kv::Record{
        name:"models".into(), //should be a string
        values: vec![
            kv::Types::String("Sony".into()), 
            kv::Types::String("Sharp".into()), 
        ]
    };


    kv::set(&bucket, "electronics-tvs", tvs.clone())?;

    let radios = kv::Record{
        name:"old-models".into(), //should be a string
        values: vec![
            kv::Types::String("Hitachi".into()), 
            kv::Types::String("Toshiba".into()), 
        ]
    };


    kv::set(&bucket, "electronics-radios", radios.clone())?;

    
    //Use Transaction to set multiple values
    // We use the convenient kv::tx! macro
    // all values are entered as key => Record
    let ts: HashMap<_, _> = kv::tx! { 
        "tx-vehicles" => vehicles ,
        "tx-radios" => radios ,
        "tx-tvs" => tvs 
    };

    //commit transaction
    kv::transaction(bucket, ts)?;

    // get single value
    let value = kv::get(&bucket, "electronics-radios");
    println!("{:#?}", value);

    // get all values with filter
    // here we filter for all electronics that are old models
    // Because the Filter struct takes Options as values, you can also enter None
    // Note: filters ise the WildMatch crate (https://docs.rs/wildmatch/2.1.0/wildmatch/) so all patterns supported by WildMatch will work fine. 
    let filter = kv::Filter{
        key : Some("electronics*"),
        name:Some("old*")
        // name:None
    };

    // now fetch
    let all = kv::get_all(&bucket, Some(filter));

    println!("{:#?}", all);

}

```