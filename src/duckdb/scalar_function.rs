use crate::duckdb::bindings::{
    duckdb_create_scalar_function, duckdb_destroy_scalar_function, duckdb_scalar_function,
    duckdb_scalar_function_add_parameter, duckdb_scalar_function_set_function,
    duckdb_scalar_function_set_name, duckdb_scalar_function_set_return_type,
    duckdb_scalar_function_t,
};
use crate::duckdb::logical_type::LogicalType;
use crate::Result;
use std::ffi::CString;

pub struct ScalarFunction(pub(in crate::duckdb) duckdb_scalar_function);

impl ScalarFunction {
    pub fn new() -> Self {
        Self(unsafe { duckdb_create_scalar_function() })
    }

    pub fn set_name(&self, name: &str) -> Result<()> {
        unsafe {
            duckdb_scalar_function_set_name(self.0, CString::new(name)?.as_ptr());
        }
        Ok(())
    }

    pub fn set_return_type(&self, ty: &LogicalType) {
        unsafe {
            duckdb_scalar_function_set_return_type(self.0, ty.0);
        }
    }

    pub fn set_function(&self, f: duckdb_scalar_function_t) {
        unsafe {
            duckdb_scalar_function_set_function(self.0, f);
        }
    }

    pub fn add_parameter(&self, ty: &LogicalType) {
        unsafe { duckdb_scalar_function_add_parameter(self.0, ty.0) }
    }
}

impl Drop for ScalarFunction {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_scalar_function(&mut self.0);
        }
    }
}
