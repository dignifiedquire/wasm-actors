#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use alloc::{format, vec};
use alloc::vec::Vec;
use core::panic::PanicInfo;

use shared::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn rt_create(ptr: i64, len: i32);
    fn write_return_bytes(ptr: i64, len: i32);
    fn dbg(i: i32);
}

fn debug(i: i32) {
    unsafe { dbg(i) };
}

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn add_one(a: i32) -> i32 {
    a.wrapping_add(1)
}

// #[no_mangle]
// pub extern "C" fn add_one_cbor(ptr: i32, len: i32) {
//     let arg: [u8; 2] = read_arguments(ptr, len);
//     let programming_languages = vec![
//         ("rust", vec!["safe", "concurrent", "fast"]),
//         ("python", vec!["powerful", "friendly", "open"]),
//         ("js", vec!["lightweight", "interpreted", "object-oriented"]),
//     ];
//     let foo = [1u8; 8];

//     unsafe {
//         put(1, foo.as_ptr() as i32, foo.len() as i32);
//     }

//     return_value(programming_languages)
// }

// WASM
//
// BUFFERS: Vec<u8>
//
// read arguments
//   - read slice based on ptr, len from WASM[BUFFERES]
//   - deserialize with cbor
//
// return values
//  - serialize into WASM[Memory]
//  - store [len, ptr] in WASM[BUFFERS]


/// Host managed memory.
static mut BUFFER: Vec<u8> = Vec::new();

/// Used by the host to allocate memory.
#[no_mangle]
pub unsafe extern "C" fn alloc_buffer(size: i32) -> i64 {
    debug(size);
    BUFFER.clear();
    BUFFER.resize(size as usize, 0);
    BUFFER.as_ptr() as i64
}

struct Runtime {}

impl Runtime {
    pub fn create<T: serde::Serialize>(value: T) {
        let bytes = serde_cbor::to_vec(&value).unwrap();
        unsafe {
            rt_create(from_u64(bytes.as_ptr() as u64), from_u32(bytes.len() as u32));
        }
    }
}

#[no_mangle]
pub extern "C" fn cool_actor_constructor(ptr: i64, len: i32) {
    debug(1);
    let args: NewParams = read_arguments(ptr, len);
    debug(2);
    let actor = CoolActor::new(args);

    debug(3);
    Runtime::create(actor);

    debug(4);
    write_return(NewReturn {});
}

fn write_return<T: serde::Serialize>(val: T) {
    let bytes = serde_cbor::to_vec(&val).unwrap();

    unsafe {
        write_return_bytes(from_u64(bytes.as_ptr() as u64), from_u32(bytes.len() as u32));
    }
}
    
fn read_arguments<T: serde::de::DeserializeOwned>(ptr: i64, len: i32) -> T {
    debug(len);

    let ptr = from_i64(ptr);
    let len = from_i32(len);
    debug(9);
    let slice: &[u8] = unsafe {
        core::slice::from_raw_parts(ptr as _, len as _)
    };
    debug(slice.len() as i32);
    let x = serde_cbor::from_slice(slice).unwrap();
    debug(11);
    x
}

// Host
// pass arguments:
//   - serialize with cbor
//   - allocate in WASM[BUFFERES]
//   - copy to WASM[BUFFERS]
//   - call function with (ptr, len) in WASM
//
// read return values
//   - read [len, ptr] from WASM[BUFFERS]
//   - read slice from WASM[Memory] based on ptr, len
//   - deserialize slice
//   - clear data from WASM[Memory]



/// Stores the return value serialiezd as cbor in memory,
// #[inline]
// fn return_value<T: serde::Serialize>(val: T) {
//     let raw = serde_cbor::to_vec(&val).unwrap();
//     unsafe { put_return_value(raw.as_ptr() as i32, raw.len() as i32); }
// }
// #[inline]
// fn read_arguments<T: serde::de::DeserializeOwned>(ptr: i32, len: i32) -> T {
//     let raw = core::slice::from_raw_parts(ptr as *const u8, len as usize);
//     serde_cbor::from_slice(raw).unwrap()
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add_one(1), 2);
    }
}
