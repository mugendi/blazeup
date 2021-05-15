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

pub mod db;

use crate::db::kv;

use std::{error::Error, path::Path, result::Result  };
// use kv::*;


fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("D:\\projects\\RUST\\blazeup\\db");



    // print

    // reset
    kv::reset(Some(&path))?;

    //init with index path...
    kv::init(Some(&path))?;

    // set bucket, can have multiple
    let bucket = "Js";

    let val = kv::Record{
        name:"beees".into(),
        values: vec![kv::Types::String("john".into()), kv::Types::I32(43)]
    };


    // set value
    kv::set(&bucket, "test", val)?;

    // get value
    let value = kv::get(&bucket, "test");
    println!("{:#?}", value);
    // get all values in a vector via the iter
    let all = kv::get_all(&bucket);

    println!("{:#?}", all);

    Ok(())
}
