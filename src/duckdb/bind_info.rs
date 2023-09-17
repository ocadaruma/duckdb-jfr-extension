use crate::duckdb::bindings::{duckdb_bind_add_result_column, duckdb_bind_get_parameter, duckdb_bind_info, duckdb_bind_set_bind_data, duckdb_delete_callback_t, duckdb_scalar_bind_info, duckdb_scalar_bind_set_bind_data};
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::value::Value;
use crate::Result;
use std::ffi::{c_void, CString};

pub struct BindInfo(duckdb_bind_info);

impl BindInfo {
    pub fn from(info: duckdb_bind_info) -> Self {
        Self(info)
    }

    pub fn get_parameter(&self, idx: usize) -> Value {
        Value::from(unsafe { duckdb_bind_get_parameter(self.0, idx as u64) })
    }

    pub fn add_result_column(&self, name: &str, ty: &LogicalType) -> Result<()> {
        unsafe {
            duckdb_bind_add_result_column(self.0, CString::new(name)?.as_ptr(), ty.ptr());
        }
        Ok(())
    }

    pub fn set_bind_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe {
            duckdb_bind_set_bind_data(self.0, data, free_function);
        }
    }
}

pub struct ScalarBindInfo(duckdb_scalar_bind_info);

impl ScalarBindInfo {
    pub fn from(info: duckdb_scalar_bind_info) -> Self {
        Self(info)
    }

    pub fn set_bind_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe {
            duckdb_scalar_bind_set_bind_data(self.0, data, free_function);
        }
    }
}
