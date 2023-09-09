#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::all)]
#[allow(improper_ctypes)]
mod bindgen {
    include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));
}

pub use bindgen::*;

pub const DuckDBError: duckdb_state = duckdb_state_DuckDBError;
pub const DuckDBSuccess: duckdb_state = duckdb_state_DuckDBSuccess;

#[repr(u32)]
pub enum LogicalTypeId {
    Boolean = DUCKDB_TYPE_DUCKDB_TYPE_BOOLEAN,
    Tinyint = DUCKDB_TYPE_DUCKDB_TYPE_TINYINT,
    Smallint = DUCKDB_TYPE_DUCKDB_TYPE_SMALLINT,
    Integer = DUCKDB_TYPE_DUCKDB_TYPE_INTEGER,
    Bigint = DUCKDB_TYPE_DUCKDB_TYPE_BIGINT,
    Utinyint = DUCKDB_TYPE_DUCKDB_TYPE_UTINYINT,
    Usmallint = DUCKDB_TYPE_DUCKDB_TYPE_USMALLINT,
    Uinteger = DUCKDB_TYPE_DUCKDB_TYPE_UINTEGER,
    Ubigint = DUCKDB_TYPE_DUCKDB_TYPE_UBIGINT,
    Float = DUCKDB_TYPE_DUCKDB_TYPE_FLOAT,
    Double = DUCKDB_TYPE_DUCKDB_TYPE_DOUBLE,
    Timestamp = DUCKDB_TYPE_DUCKDB_TYPE_TIMESTAMP,
    Date = DUCKDB_TYPE_DUCKDB_TYPE_DATE,
    Time = DUCKDB_TYPE_DUCKDB_TYPE_TIME,
    Interval = DUCKDB_TYPE_DUCKDB_TYPE_INTERVAL,
    Hugeint = DUCKDB_TYPE_DUCKDB_TYPE_HUGEINT,
    Varchar = DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR,
    Blob = DUCKDB_TYPE_DUCKDB_TYPE_BLOB,
    Decimal = DUCKDB_TYPE_DUCKDB_TYPE_DECIMAL,
    TimestampS = DUCKDB_TYPE_DUCKDB_TYPE_TIMESTAMP_S,
    TimestampMs = DUCKDB_TYPE_DUCKDB_TYPE_TIMESTAMP_MS,
    TimestampNs = DUCKDB_TYPE_DUCKDB_TYPE_TIMESTAMP_NS,
    Enum = DUCKDB_TYPE_DUCKDB_TYPE_ENUM,
    List = DUCKDB_TYPE_DUCKDB_TYPE_LIST,
    Struct = DUCKDB_TYPE_DUCKDB_TYPE_STRUCT,
    Map = DUCKDB_TYPE_DUCKDB_TYPE_MAP,
    Uuid = DUCKDB_TYPE_DUCKDB_TYPE_UUID,
    Union = DUCKDB_TYPE_DUCKDB_TYPE_UNION,
}

/// Open file with read access
pub const FILE_FLAGS_READ: u8 = 1 << 0;
/// Open file with write access
pub const FILE_FLAGS_WRITE: u8 = 1 << 1;
/// Use direct IO when reading/writing to the file
pub const FILE_FLAGS_DIRECT_IO: u8 = 1 << 2;
/// Create file if not exists, can only be used together with WRITE
pub const FILE_FLAGS_FILE_CREATE: u8 = 1 << 3;
/// Always create a new file. If a file exists, the file is truncated. Cannot be used together with CREATE.
pub const FILE_FLAGS_FILE_CREATE_NEW: u8 = 1 << 4;
/// Open file in append mode
pub const FILE_FLAGS_APPEND: u8 = 1 << 5;
