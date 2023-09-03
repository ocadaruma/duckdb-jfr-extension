use crate::duckdb::logical_type::LogicalType;
use crate::Result;
use libduckdb_sys::{
    duckdb_bind_add_result_column, duckdb_bind_info, duckdb_bind_set_bind_data,
    duckdb_delete_callback_t, duckdb_init_info, duckdb_init_set_init_data,
};
use std::ffi::{c_void, CString};

pub struct InitInfo(duckdb_init_info);

impl InitInfo {
    pub fn from(info: duckdb_init_info) -> Self {
        Self(info)
    }

    pub fn add_result_column(&self, name: &str, ty: &LogicalType) -> Result<()> {
        unsafe {
            duckdb_bind_add_result_column(self.0, CString::new(name)?.as_ptr(), ty.ptr());
        }
        Ok(())
    }

    pub fn set_init_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe {
            duckdb_init_set_init_data(self.0, data, free_function);
        }
    }
}
