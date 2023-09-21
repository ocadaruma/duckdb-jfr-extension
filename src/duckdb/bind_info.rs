use crate::duckdb::bindings::{
    duckdb_bind_add_result_column, duckdb_bind_get_parameter, duckdb_bind_info,
    duckdb_bind_set_bind_data,
};
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::value::Value;
use crate::Result;
use std::ffi::{c_void, CString};

pub struct BindInfo(duckdb_bind_info);

impl BindInfo {
    pub fn from_ptr(info: duckdb_bind_info) -> Self {
        Self(info)
    }

    pub fn get_parameter(&self, idx: usize) -> Value {
        Value(unsafe { duckdb_bind_get_parameter(self.0, idx as u64) })
    }

    pub fn add_result_column(&self, name: &str, ty: &LogicalType) -> Result<()> {
        unsafe {
            duckdb_bind_add_result_column(self.0, CString::new(name)?.as_ptr(), ty.0);
        }
        Ok(())
    }

    pub fn set_bind_data<T>(&self, data: Box<T>) {
        unsafe {
            duckdb_bind_set_bind_data(self.0, Box::into_raw(data).cast(), Some(Self::free::<T>));
        }
    }

    extern "C" fn free<T>(ptr: *mut c_void) {
        unsafe {
            let _ = Box::<T>::from_raw(ptr.cast());
        }
    }
}
