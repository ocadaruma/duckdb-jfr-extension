use crate::duckdb::bindings::{duckdb_init_info, duckdb_init_set_init_data};

use std::ffi::c_void;

pub struct InitInfo(duckdb_init_info);

impl InitInfo {
    pub fn from_ptr(info: duckdb_init_info) -> Self {
        Self(info)
    }

    pub fn set_init_data<T>(&self, data: Box<T>) {
        unsafe {
            duckdb_init_set_init_data(self.0, Box::into_raw(data).cast(), Some(Self::free::<T>));
        }
    }

    extern "C" fn free<T>(ptr: *mut c_void) {
        unsafe {
            let _ = Box::<T>::from_raw(ptr.cast());
        }
    }
}
