use crate::duckdb::bindings::{
    duckdb_client_context, duckdb_create_table_function_view, duckdb_destroy_table_function_view,
    duckdb_register_table_function_view, duckdb_table_function_view,
    duckdb_table_function_view_add_parameter, duckdb_table_function_view_set_function_name,
    duckdb_table_function_view_set_name, DuckDBSuccess,
};
use crate::duckdb::value::Value;
use crate::Result;
use anyhow::anyhow;
use std::ffi::CString;

pub struct TableFunctionView(duckdb_table_function_view);

impl TableFunctionView {
    pub fn new() -> Self {
        Self(unsafe { duckdb_create_table_function_view() })
    }

    pub fn set_function_name(&self, function_name: &str) -> Result<()> {
        unsafe {
            duckdb_table_function_view_set_function_name(
                self.0,
                CString::new(function_name)?.as_ptr(),
            );
        }
        Ok(())
    }

    pub fn set_name(&self, name: &str) -> Result<()> {
        unsafe {
            duckdb_table_function_view_set_name(self.0, CString::new(name)?.as_ptr());
        }
        Ok(())
    }

    pub fn add_parameter(&self, v: &Value) {
        unsafe { duckdb_table_function_view_add_parameter(self.0, v.0) }
    }

    pub fn register(&self, context: duckdb_client_context) -> Result<()> {
        let r = unsafe { duckdb_register_table_function_view(context, self.0) };
        if r == DuckDBSuccess {
            Ok(())
        } else {
            Err(anyhow!("Error registering table function view"))
        }
    }
}

impl Drop for TableFunctionView {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_table_function_view(&mut self.0);
        }
    }
}
