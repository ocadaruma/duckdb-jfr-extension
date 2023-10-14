use crate::duckdb::bindings::{
    duckdb_create_list_type, duckdb_create_logical_type, duckdb_create_struct_type2,
    duckdb_destroy_logical_type, duckdb_logical_type, duckdb_type, LogicalTypeId,
};
use crate::Result;
use std::ffi::CString;

pub struct LogicalType(pub(in crate::duckdb) duckdb_logical_type);

impl LogicalType {
    pub fn new(ty: LogicalTypeId) -> Self {
        Self(unsafe { duckdb_create_logical_type(ty as duckdb_type) })
    }

    pub fn new_list_type(child_type: &LogicalType) -> Self {
        Self(unsafe { duckdb_create_list_type(child_type.0) })
    }

    pub fn new_struct_type(shape: &[(&str, LogicalType)]) -> Result<Self> {
        let (mut keys, mut values) = (vec![], vec![]);
        for (key, value) in shape {
            keys.push(CString::new(*key)?);
            values.push(value.0);
        }
        let key_ptrs: Vec<_> = keys.iter().map(|s| s.as_ptr()).collect();

        Ok(Self(unsafe {
            duckdb_create_struct_type2(
                keys.len() as u64,
                key_ptrs.as_slice().as_ptr().cast_mut(),
                values.as_slice().as_ptr(),
            )
        }))
    }
}

impl Drop for LogicalType {
    fn drop(&mut self) {
        unsafe {
            duckdb_destroy_logical_type(&mut self.0);
        }
    }
}
