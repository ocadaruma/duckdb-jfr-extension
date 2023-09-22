use crate::duckdb::bindings::{
    duckdb_create_varchar_length, duckdb_destroy_value, duckdb_free, duckdb_get_varchar,
    duckdb_value, idx_t,
};
use crate::Result;
use anyhow::anyhow;
use std::ffi::{c_char, CStr, CString};

pub struct Value(pub(in crate::duckdb) duckdb_value);

impl Value {
    pub fn new_varchar(s: &str) -> Result<Self> {
        Ok(Self(unsafe {
            duckdb_create_varchar_length(CString::new(s)?.as_ptr(), s.len() as idx_t)
        }))
    }

    pub fn get_varchar(&self) -> ValueVarchar {
        ValueVarchar::new(unsafe { duckdb_get_varchar(self.0) })
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_value(&mut self.0);
        }
    }
}

pub struct ValueVarchar(*mut c_char);

impl ValueVarchar {
    pub fn new(ptr: *mut c_char) -> Self {
        Self(ptr)
    }

    pub fn as_str(&self) -> Result<&str> {
        unsafe { CStr::from_ptr(self.0).to_str().map_err(|e| anyhow!(e)) }
    }
}

impl Drop for ValueVarchar {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                duckdb_free(self.0.cast());
            }
        }
    }
}
