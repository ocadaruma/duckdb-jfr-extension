use crate::duckdb::bindings::{
    duckdb_list_vector_get_child, duckdb_list_vector_reserve, duckdb_list_vector_set_size,
    duckdb_struct_vector_get_child, duckdb_validity_set_row_invalid, duckdb_vector,
    duckdb_vector_assign_string_element_len, duckdb_vector_ensure_validity_writable,
    duckdb_vector_get_data, duckdb_vector_get_validity, idx_t,
};
use std::ffi::c_char;

pub struct Vector(duckdb_vector);

impl Vector {
    pub fn from_ptr(ptr: duckdb_vector) -> Self {
        Self(ptr)
    }

    pub fn get_struct_child(&self, index: usize) -> Self {
        Self(unsafe { duckdb_struct_vector_get_child(self.0, index as idx_t) })
    }

    pub fn get_list_child(&self) -> Self {
        Self(unsafe { duckdb_list_vector_get_child(self.0) })
    }

    pub fn reserve_list_capacity(&self, capacity: usize) {
        unsafe {
            duckdb_list_vector_reserve(self.0, capacity as idx_t);
        }
    }

    pub fn set_list_size(&self, size: usize) {
        unsafe {
            duckdb_list_vector_set_size(self.0, size as idx_t);
        }
    }

    pub fn set_null(&self, index: usize) {
        unsafe {
            duckdb_vector_ensure_validity_writable(self.0);
            duckdb_validity_set_row_invalid(duckdb_vector_get_validity(self.0), index as idx_t);
        }
    }

    pub fn get_data<T>(&self) -> *mut T {
        unsafe { duckdb_vector_get_data(self.0).cast() }
    }

    pub fn set_data<T>(&self, index: usize, value: T) {
        unsafe {
            let data = self.get_data::<T>();
            data.add(index).write(value);
        }
    }

    pub fn assign_string_element_len(&self, index: usize, s: *const c_char, len: usize) {
        unsafe { duckdb_vector_assign_string_element_len(self.0, index as idx_t, s, len as idx_t) }
    }
}
