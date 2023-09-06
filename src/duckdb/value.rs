use crate::Result;
use anyhow::anyhow;
use libduckdb_sys::{duckdb_destroy_value, duckdb_get_varchar, duckdb_value};
use std::ffi::CStr;

pub struct Value(duckdb_value);

impl Value {
    pub fn from(ptr: duckdb_value) -> Self {
        Self(ptr)
    }

    pub fn get_varchar(&self) -> Result<&str> {
        unsafe {
            CStr::from_ptr(duckdb_get_varchar(self.0))
                .to_str()
                .map_err(|e| anyhow!(e))
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_value(&mut self.0);
        }
    }
}
