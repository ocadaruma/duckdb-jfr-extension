use crate::duckdb::vector::Vector;
use libduckdb_sys::{
    duckdb_data_chunk, duckdb_data_chunk_get_column_count, duckdb_data_chunk_get_vector,
    duckdb_data_chunk_set_size, idx_t,
};

pub struct DataChunk(duckdb_data_chunk);

impl DataChunk {
    pub fn from(ptr: duckdb_data_chunk) -> Self {
        Self(ptr)
    }

    pub fn ptr(&self) -> duckdb_data_chunk {
        self.0
    }

    pub fn get_vector(&self, column_index: usize) -> Vector {
        Vector::from(unsafe { duckdb_data_chunk_get_vector(self.0, column_index as idx_t) })
    }

    pub fn set_size(&self, size: usize) {
        unsafe { duckdb_data_chunk_set_size(self.0, size as idx_t) };
    }

    pub fn get_column_count(&self) -> usize {
        unsafe { duckdb_data_chunk_get_column_count(self.0) as usize }
    }
}
