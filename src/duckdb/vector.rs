use crate::duckdb::bindings::{
    duckdb_vector, duckdb_vector_assign_string_element_len, duckdb_vector_get_data, idx_t,
};
use std::ffi::c_char;

pub struct Vector(duckdb_vector);

impl Vector {
    pub fn from(ptr: duckdb_vector) -> Self {
        Self(ptr)
    }

    pub fn ptr(&self) -> duckdb_vector {
        self.0
    }

    pub fn get_data<T>(&self) -> *mut T {
        unsafe { duckdb_vector_get_data(self.0).cast() }
    }

    pub fn assign_string_element_len(&self, index: usize, s: *const c_char, len: usize) {
        unsafe { duckdb_vector_assign_string_element_len(self.0, index as idx_t, s, len as idx_t) }
    }
}
