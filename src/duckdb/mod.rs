//! Rust API for DuckDB.

pub mod bind_info;
pub mod bindings;
pub mod data_chunk;
pub mod function_info;
pub mod init_info;
pub mod logical_type;
pub mod table_function;
pub mod value;
pub mod vector;
pub mod file;

use crate::duckdb::table_function::TableFunction;
use crate::Result;
use anyhow::anyhow;
use libduckdb_sys::{
    duckdb_connect, duckdb_connection, duckdb_database, duckdb_disconnect,
    duckdb_register_table_function, DuckDBSuccess, Error,
};
use std::ffi::c_void;
use std::ptr::null_mut;
use crate::duckdb::bindings::duckdb_register_table_function2;

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
            Err(anyhow!(Error::new(r)))
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
            Err(anyhow!(Error::new(r)))
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