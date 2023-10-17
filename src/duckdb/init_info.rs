use crate::duckdb::bindings::{
    duckdb_init_get_column_count, duckdb_init_get_column_index, duckdb_init_info,
    duckdb_init_set_init_data, idx_t,
};

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

    pub fn projected_column_count(&self) -> usize {
        unsafe { duckdb_init_get_column_count(self.0) as usize }
    }

    pub fn column_index(&self, projection_idx: usize) -> usize {
        unsafe { duckdb_init_get_column_index(self.0, projection_idx as idx_t) as usize }
    }

    extern "C" fn free<T>(ptr: *mut c_void) {
        unsafe {
            let _ = Box::<T>::from_raw(ptr.cast());
        }
    }
}
