use crate::duckdb::bindings::{duckdb_create_scalar_function, duckdb_create_table_function2, duckdb_destroy_scalar_function, duckdb_scalar_function_add_parameter, duckdb_scalar_function, duckdb_scalar_function_set_function, duckdb_scalar_function_t, duckdb_table_function2_bind_t, duckdb_table_function2_set_bind, duckdb_table_function2_set_function, duckdb_table_function2_set_init, duckdb_table_function2_t};
use crate::duckdb::logical_type::LogicalType;
use crate::Result;
use libduckdb_sys::{
    duckdb_destroy_table_function, duckdb_table_function, duckdb_table_function_add_parameter, duckdb_table_function_init_t, duckdb_table_function_set_name,
};
use std::ffi::CString;

pub struct ScalarFunction(duckdb_scalar_function);

impl ScalarFunction {
    pub fn new(name: &str, return_type: &LogicalType) -> Result<Self> {
        Ok(Self(unsafe { duckdb_create_scalar_function(
            CString::new(name)?.as_ptr(), return_type.ptr()) }))
    }

    pub fn ptr(&self) -> duckdb_scalar_function {
        self.0
    }

    pub fn set_function(&self, f: duckdb_scalar_function_t) {
        unsafe {
            duckdb_scalar_function_set_function(self.0, f);
        }
    }

    pub fn add_parameter(&self, ty: &LogicalType) {
        unsafe { duckdb_scalar_function_add_parameter(self.0, ty.ptr()) }
    }
}

impl Drop for ScalarFunction {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_scalar_function(&mut self.0);
        }
    }
}
