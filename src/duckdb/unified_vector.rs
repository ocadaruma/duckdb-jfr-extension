use crate::duckdb::bindings::{
    duckdb_destroy_unified_vector_format, duckdb_get_string, duckdb_to_unified_format,
    duckdb_unified_vector_format, duckdb_unified_vector_validity_row_is_valid, duckdb_vector,
    idx_t, string_piece,
};

pub struct UnifiedVector(duckdb_unified_vector_format);

impl UnifiedVector {
    pub fn new(vector: duckdb_vector, count: idx_t) -> Self {
        Self(unsafe { duckdb_to_unified_format(vector, count) })
    }

    pub fn ptr(&self) -> duckdb_unified_vector_format {
        self.0
    }

    pub fn get_string(&self, index: idx_t) -> string_piece {
        unsafe { duckdb_get_string(self.0, index) }
    }

    pub fn is_null(&self, index: idx_t) -> bool {
        unsafe { duckdb_unified_vector_validity_row_is_valid(self.0, index) }
    }
}

impl Drop for UnifiedVector {
    fn drop(&mut self) {
        unsafe { duckdb_destroy_unified_vector_format(&mut self.0) }
    }
}
