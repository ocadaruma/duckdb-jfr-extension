use libduckdb_sys::{duckdb_function_get_bind_data, duckdb_function_get_extra_info, duckdb_function_get_init_data, duckdb_function_info};
use std::ffi::c_void;
use crate::duckdb::bindings::{duckdb_function2_get_bind_data, duckdb_function2_get_init_data};

pub struct FunctionInfo(duckdb_function_info);

impl FunctionInfo {
    pub fn from(info: duckdb_function_info) -> Self {
        Self(info)
    }

    pub fn get_bind_data<T>(&self) -> *mut T {
        unsafe { duckdb_function2_get_bind_data(self.0).cast() }
    }

    pub fn get_init_data<T>(&self) -> *mut T {
        unsafe { duckdb_function2_get_init_data(self.0).cast() }
    }
}