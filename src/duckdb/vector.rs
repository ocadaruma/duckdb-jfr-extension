use crate::duckdb::bindings::{
    duckdb_list_vector_get_child, duckdb_struct_vector_get_child, duckdb_to_unified_format,
    duckdb_validity_set_row_invalid, duckdb_vector, duckdb_vector_assign_string_element_len,
    duckdb_vector_ensure_validity_writable, duckdb_vector_get_data, duckdb_vector_get_validity,
    duckdb_vector_is_constant, idx_t,
};
use crate::duckdb::unified_vector::UnifiedVector;
use std::ffi::c_char;

pub struct Vector(duckdb_vector);

impl Vector {
    pub fn from_ptr(ptr: duckdb_vector) -> Self {
        Self(ptr)
    }

    pub fn to_unified_format(&self, count: idx_t) -> UnifiedVector {
        UnifiedVector(unsafe { duckdb_to_unified_format(self.0, count) })
    }

    pub fn get_struct_child(&self, index: idx_t) -> Self {
        Self(unsafe { duckdb_struct_vector_get_child(self.0, index) })
    }

    pub fn get_list_child(&self) -> Self {
        Self(unsafe { duckdb_list_vector_get_child(self.0) })
    }

    pub fn is_constant(&self) -> bool {
        unsafe { duckdb_vector_is_constant(self.0) }
    }

    pub fn set_null(&self, index: idx_t) {
        unsafe {
            duckdb_vector_ensure_validity_writable(self.0);
            duckdb_validity_set_row_invalid(duckdb_vector_get_validity(self.0), index);
        }
    }

    pub fn get_data<T>(&self) -> *mut T {
        unsafe { duckdb_vector_get_data(self.0).cast() }
    }

    pub fn assign_string_element_len(&self, index: usize, s: *const c_char, len: usize) {
        unsafe { duckdb_vector_assign_string_element_len(self.0, index as idx_t, s, len as idx_t) }
    }
}
