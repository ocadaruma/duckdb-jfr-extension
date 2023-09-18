//! Rust API for DuckDB.

pub mod bind_info;
pub mod bindings;
pub mod data_chunk;
pub mod file;
pub mod function_info;
pub mod init_info;
pub mod logical_type;
pub mod scalar_function;
pub mod table_function;
pub mod unified_vector;
pub mod value;
pub mod vector;

use crate::duckdb::bindings::{
    duckdb_connect, duckdb_connection, duckdb_database, duckdb_disconnect, duckdb_malloc,
    duckdb_register_scalar_function, duckdb_register_table_function2, DuckDBSuccess,
};
use crate::duckdb::scalar_function::ScalarFunction;
use crate::duckdb::table_function::TableFunction;
use crate::Result;
use anyhow::anyhow;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;

pub struct Database(duckdb_database);

impl Database {
    pub fn from(ptr: *mut c_void) -> Self {
        Self(ptr.cast())
    }

    pub fn ptr(&self) -> duckdb_database {
        self.0
    }

    pub fn connect(&self) -> Result<Connection> {
        let mut conn = null_mut();
        let r = unsafe { duckdb_connect(self.0, &mut conn) };
        if r == DuckDBSuccess {
            Ok(Connection(conn))
        } else {
            Err(anyhow!("Error connecting to database"))
        }
    }
}

pub struct Connection(duckdb_connection);

impl Connection {
    pub fn ptr(&self) -> duckdb_connection {
        self.0
    }

    pub fn register_table_function(&self, f: &TableFunction) -> Result<()> {
        let r = unsafe { duckdb_register_table_function2(self.0, f.ptr()) };
        if r == DuckDBSuccess {
            Ok(())
        } else {
            Err(anyhow!("Error registering table function"))
        }
    }

    pub fn register_scalar_function(&self, f: &ScalarFunction) -> Result<()> {
        let r = unsafe { duckdb_register_scalar_function(self.0, f.ptr()) };
        if r == DuckDBSuccess {
            Ok(())
        } else {
            Err(anyhow!("Error registering scalar function"))
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { duckdb_disconnect(&mut self.0) }
        }
    }
}

pub unsafe fn malloc_struct<T>() -> *mut T {
    duckdb_malloc(size_of::<T>()).cast::<T>()
}
