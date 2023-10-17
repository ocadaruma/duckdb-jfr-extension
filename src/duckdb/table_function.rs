use crate::duckdb::bindings::{
    duckdb_create_table_function2, duckdb_destroy_table_function, duckdb_table_function,
    duckdb_table_function2_bind_t, duckdb_table_function2_set_bind,
    duckdb_table_function2_set_function, duckdb_table_function2_set_init, duckdb_table_function2_t,
    duckdb_table_function_add_parameter, duckdb_table_function_init_t,
    duckdb_table_function_set_name, duckdb_table_function_supports_projection_pushdown,
};
use crate::duckdb::logical_type::LogicalType;
use crate::Result;
use std::ffi::CString;

pub struct TableFunction(pub(in crate::duckdb) duckdb_table_function);

impl TableFunction {
    pub fn new() -> Self {
        Self(unsafe { duckdb_create_table_function2() })
    }

    pub fn set_name(&self, name: &str) -> Result<()> {
        unsafe {
            duckdb_table_function_set_name(self.0, CString::new(name)?.as_ptr());
        }
        Ok(())
    }

    pub fn set_supports_projection_pushdown(&self, supports: bool) {
        unsafe {
            duckdb_table_function_supports_projection_pushdown(self.0, supports);
        }
    }

    pub fn set_function(&self, f: duckdb_table_function2_t) {
        unsafe {
            duckdb_table_function2_set_function(self.0, f);
        }
    }

    pub fn set_init(&self, f: duckdb_table_function_init_t) {
        unsafe {
            duckdb_table_function2_set_init(self.0, f);
        }
    }

    pub fn set_bind(&self, f: duckdb_table_function2_bind_t) {
        unsafe {
            duckdb_table_function2_set_bind(self.0, f);
        }
    }

    pub fn add_parameter(&self, ty: &LogicalType) {
        unsafe { duckdb_table_function_add_parameter(self.0, ty.0) }
    }
}

impl Drop for TableFunction {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_table_function(&mut self.0);
        }
    }
}
