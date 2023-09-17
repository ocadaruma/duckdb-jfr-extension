use crate::duckdb::bindings::{
    duckdb_delete_callback_t, duckdb_function2_get_bind_data, duckdb_function2_get_init_data,
    duckdb_function_info, duckdb_scalar_function_get_bind_data,
    duckdb_scalar_function_get_init_data, duckdb_scalar_function_info,
    duckdb_scalar_function_set_bind_data, duckdb_scalar_function_set_init_data,
};
use std::ffi::c_void;

pub struct FunctionInfo(duckdb_function_info);

impl FunctionInfo {
    pub fn from(info: duckdb_function_info) -> Self {
        Self(info)
    }

    pub fn get_bind_data<T>(&self) -> *mut T {
        unsafe { duckdb_function2_get_bind_data(self.0).cast() }
    }

    pub fn get_init_data<T>(&self) -> *mut T {
        unsafe { duckdb_function2_get_init_data(self.0).cast() }
    }
}

pub struct ScalarFunctionInfo(duckdb_scalar_function_info);

impl ScalarFunctionInfo {
    pub fn from(info: duckdb_scalar_function_info) -> Self {
        Self(info)
    }

    pub fn set_bind_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe { duckdb_scalar_function_set_bind_data(self.0, data, free_function) }
    }

    pub fn set_init_data(&self, data: *mut c_void, free_function: duckdb_delete_callback_t) {
        unsafe { duckdb_scalar_function_set_init_data(self.0, data, free_function) }
    }

    pub fn get_bind_data<T>(&self) -> *mut T {
        unsafe { duckdb_scalar_function_get_bind_data(self.0).cast() }
    }

    pub fn get_init_data<T>(&self) -> *mut T {
        unsafe { duckdb_scalar_function_get_init_data(self.0).cast() }
    }
}
