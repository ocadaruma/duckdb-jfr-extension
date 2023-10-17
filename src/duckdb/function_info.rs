use crate::duckdb::bindings::{
    duckdb_function2_get_bind_data, duckdb_function2_get_init_data, duckdb_function_info,
};
use std::mem::ManuallyDrop;

pub struct FunctionInfo(duckdb_function_info);

impl FunctionInfo {
    pub fn from_ptr(info: duckdb_function_info) -> Self {
        Self(info)
    }

    pub fn get_bind_data<T>(&self) -> Option<ManuallyDrop<Box<T>>> {
        let ptr: *mut T = unsafe { duckdb_function2_get_bind_data(self.0).cast() };
        if ptr.is_null() {
            None
        } else {
            Some(ManuallyDrop::new(unsafe { Box::from_raw(ptr) }))
        }
    }

    pub fn get_init_data<T>(&self) -> Option<ManuallyDrop<Box<T>>> {
        let ptr: *mut T = unsafe { duckdb_function2_get_init_data(self.0).cast() };
        if ptr.is_null() {
            None
        } else {
            Some(ManuallyDrop::new(unsafe { Box::from_raw(ptr) }))
        }
    }
}
