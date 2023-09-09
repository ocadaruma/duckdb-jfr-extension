use crate::duckdb::bindings::{
    duckdb_bind_add_result_column, duckdb_delete_callback_t, duckdb_init_info,
    duckdb_init_set_init_data,
};
use crate::duckdb::logical_type::LogicalType;
use crate::Result;
use std::ffi::{c_void, CString};

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
