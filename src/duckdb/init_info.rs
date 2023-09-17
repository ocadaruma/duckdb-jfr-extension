use crate::duckdb::bindings::{duckdb_delete_callback_t, duckdb_init_info, duckdb_init_set_init_data, duckdb_scalar_init_info, duckdb_scalar_init_set_init_data};

use std::ffi::c_void;

pub struct InitInfo(duckdb_init_info);

impl InitInfo {
    pub fn from(info: duckdb_init_info) -> Self {
        Self(info)
    }

    pub fn set_init_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe {
            duckdb_init_set_init_data(self.0, data, free_function);
        }
    }
}

pub struct ScalarInitInfo(duckdb_scalar_init_info);

impl ScalarInitInfo {
    pub fn from(info: duckdb_scalar_init_info) -> Self {
        Self(info)
    }

    pub fn set_init_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe {
            duckdb_scalar_init_set_init_data(self.0, data, free_function);
        }
    }
}
