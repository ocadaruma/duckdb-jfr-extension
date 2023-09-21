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
    duckdb_close, duckdb_connect, duckdb_connection, duckdb_database, duckdb_disconnect,
    duckdb_open, duckdb_register_scalar_function, duckdb_register_table_function2, DuckDBSuccess,
};
use crate::duckdb::scalar_function::ScalarFunction;
use crate::duckdb::table_function::TableFunction;
use crate::Result;
use anyhow::anyhow;
use std::ffi::CString;
use std::ptr::null_mut;

pub struct Database(Ownership);

enum Ownership {
    Owned(duckdb_database),
    Borrowed(duckdb_database),
}

impl Ownership {
    fn ptr(&self) -> duckdb_database {
        match self {
            Ownership::Owned(ptr) => *ptr,
            Ownership::Borrowed(ptr) => *ptr,
        }
    }
}

impl Database {
    pub fn new_in_memory() -> Result<Self> {
        let mut db: duckdb_database = null_mut();
        let filename = CString::new(":memory:")?;
        let r = unsafe { duckdb_open(filename.as_ptr(), &mut db) };
        if r == DuckDBSuccess {
            Ok(Self(Ownership::Owned(db)))
        } else {
            Err(anyhow!("Error opening database"))
        }
    }

    pub fn from_ptr(ptr: duckdb_database) -> Self {
        Self(Ownership::Borrowed(ptr.cast()))
    }

    /// Visible for testing
    pub(crate) fn ptr(&self) -> duckdb_database {
        self.0.ptr()
    }

    pub fn connect(&self) -> Result<Connection> {
        let mut conn = null_mut();
        let r = unsafe { duckdb_connect(self.0.ptr(), &mut conn) };
        if r == DuckDBSuccess {
            Ok(Connection(conn))
        } else {
            Err(anyhow!("Error connecting to database"))
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if let Ownership::Owned(mut db) = self.0 {
            if !db.is_null() {
                unsafe { duckdb_close(&mut db) }
            }
        }
    }
}

pub struct Connection(duckdb_connection);

impl Connection {
    pub fn register_table_function(&self, f: &TableFunction) -> Result<()> {
        let r = unsafe { duckdb_register_table_function2(self.0, f.0) };
        if r == DuckDBSuccess {
            Ok(())
        } else {
            Err(anyhow!("Error registering table function"))
        }
    }

    pub fn register_scalar_function(&self, f: &ScalarFunction) -> Result<()> {
        let r = unsafe { duckdb_register_scalar_function(self.0, f.0) };
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
