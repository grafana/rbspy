#![cfg_attr(rustc_nightly, feature(test))]

#[cfg(test)]
extern crate byteorder;
extern crate chrono;
extern crate clap;
extern crate elf;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate libc;
#[cfg(target_os = "macos")]
extern crate libproc;
#[cfg(unix)]
extern crate proc_maps;
#[macro_use]
extern crate log;
extern crate rand;
#[cfg(test)]
extern crate rbspy_testdata;
extern crate remoteprocess;

extern crate rbspy_ruby_structs as bindings;
#[cfg(windows)]
extern crate winapi;


pub mod core;

use crate::core::types::Pid;
use crate::core::initialize::initialize;
use crate::core::initialize::StackTraceGetter;

use std::env;
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

fn copy_error(err_ptr: *mut u8, err_len: i32, err_str: String) -> i32 {
    let slice = err_str.as_bytes();
    let l = slice.len();
    if l as i32 > err_len {
        return copy_error(err_ptr, err_len, "error buffer is too small".to_string());
    }
    let target = unsafe { slice::from_raw_parts_mut(err_ptr, l as usize) };
    target.clone_from_slice(slice);
    -(l as i32)
}

#[no_mangle]
pub extern "C" fn rbspy_init(pid: Pid, blocking: i32, err_ptr: *mut u8, err_len: i32) -> i32 {
    match initialize(pid, blocking != 0) {
        Ok(getter) => {
            let mut map = HASHMAP.lock().unwrap(); // get()
            map.insert(pid, getter);
            1
        }
        Err(err) => {
            copy_error(err_ptr, err_len, err.to_string())
        }
    }
}

#[no_mangle]
pub extern "C" fn rbspy_cleanup(pid: Pid, _err_ptr: *mut u8, _err_len: i32) -> i32 {
    let mut map = HASHMAP.lock().unwrap();
    map.remove(&pid);
    1
}

#[no_mangle]
pub extern "C" fn rbspy_snapshot(pid: Pid, ptr: *mut u8, len: i32, err_ptr: *mut u8, err_len: i32) -> i32 {
    let mut map = HASHMAP.lock().unwrap(); // get()

    let cwd = env::current_dir().unwrap();
    let cwd = cwd.to_str().unwrap_or("");

    match map.get_mut(&pid) {
        Some(getter) => {
            let mut res = 0;
            match getter.get_trace() {
                Ok(trace2) => {
                    match trace2 {
                        Some(trace) => {
                            // if trace.on_cpu != Some(true) {
                            //     res = copy_error(err_ptr, err_len, "not on cpu".to_string())
                            // } else {
                            let mut string_list = vec![];
                            for x in trace.iter().rev() {
                                let mut s = x.to_string();

                                // TODO: there must be a way to write this cleanly
                                match s.find(cwd) {
                                    Some(i) => {
                                        s = s[(i+cwd.len()+1)..].to_string();
                                    }
                                    None => {
                                        match s.find("/gems/") {
                                            Some(i) => {
                                                s = s[(i+1)..].to_string();
                                            }
                                            None => {
                                                match s.find("/ruby/") {
                                                    Some(i) => {
                                                        s = s[(i+6)..].to_string();
                                                        match s.find("/") {
                                                            Some(i) => {
                                                                s = s[(i+1)..].to_string();
                                                            }
                                                            None => {
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                string_list.push(s);
                            }
                            let joined = string_list.join(";");
                            let joined_slice = joined.as_bytes();
                            let l = joined_slice.len();

                            if len < (l as i32) {
                                res = copy_error(err_ptr, err_len, "buffer is too small".to_string())
                            } else {
                                let slice = unsafe { slice::from_raw_parts_mut(ptr, l as usize) };
                                slice.clone_from_slice(joined_slice);
                                res = l as i32
                            }
                        }
                        None => {
                            res = copy_error(err_ptr, err_len, "failure".to_string())
                        }
                    }
                }
                Err(err) => {
                    res = copy_error(err_ptr, err_len, err.to_string())
                }
            }
            res
        }
        None => copy_error(err_ptr, err_len, "could not find spy for this pid".to_string())
    }
}
