#![cfg_attr(rustc_nightly, feature(test))]

#[cfg(test)]
extern crate byteorder;
extern crate chrono;
extern crate clap;
extern crate elf;
extern crate env_logger;
extern crate inferno;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate libc;
#[cfg(target_os = "macos")]
extern crate libproc;
#[cfg(unix)]
extern crate nix;
extern crate proc_maps;
#[macro_use]
extern crate log;
extern crate rand;
#[cfg(test)]
extern crate rbspy_testdata;
extern crate remoteprocess;

extern crate rbspy_ruby_structs as bindings;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tempdir;
extern crate term_size;
#[cfg(windows)]
extern crate winapi;


pub mod core;

use crate::core::types::Pid;
use crate::core::initialize::initialize;
use crate::core::initialize::StackTraceGetter;

use libc::*;
use std::panic::*;
use std::ptr::*;

use std::slice;

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<Pid, StackTraceGetter>> =
    {
        let h = HashMap::new();
        Mutex::new(h)
    };
}

#[no_mangle]
pub extern "C" fn rbspy_init(pid: Pid) -> i32 {
    match initialize(pid) {
        Ok(getter) => {
            let mut map = HASHMAP.lock().unwrap(); // get()
            map.insert(pid, getter);
        }
        Err(err) => {
            // TODO: return error string
            return -1
        }
    }

    return 1
}

#[no_mangle]
pub extern "C" fn rbspy_cleanup(pid: Pid) -> i32 {
    let mut map = HASHMAP.lock().unwrap(); // get()
    map.remove(&pid);
    1
}

#[no_mangle]
pub extern "C" fn rbspy_snapshot(pid: Pid, ptr: *mut u8, len: i32) -> i32 {
    let mut map = HASHMAP.lock().unwrap(); // get()
    match map.get_mut(&pid) {
        Some(getter) => {
            let mut res = 0;
            match getter.get_trace() {
              Ok(trace) => {
                let mut string_list = vec![];
                for x in trace.iter().rev() {
                    string_list.push(x.to_string());
                }
                let joined = string_list.join(";");
                let joined_slice = joined.as_bytes();
                let l = joined_slice.len();

                if len < (l as i32) {
                    res = -1;
                } else {
                    let slice = unsafe { slice::from_raw_parts_mut(ptr, l as usize) };
                    slice.clone_from_slice(joined_slice);
                    res = l as i32
                }
              }
              Err(err) => {
                  res = -3
              }
            }
            res
        }
        None => -2
    }
}
