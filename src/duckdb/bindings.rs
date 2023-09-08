#![allow(non_camel_case_types)]

use libduckdb_sys::*;
use std::ffi::{c_char, c_void};

pub type duckdb_file_handle = *mut c_void;
pub type duckdb_client_context = *mut c_void;
pub type duckdb_scalar_function = *mut c_void;
pub type duckdb_expression_state = *mut c_void;
pub type duckdb_table_function2_bind_t =
    Option<unsafe extern "C" fn(ctx: duckdb_client_context, info: duckdb_bind_info)>;
pub type duckdb_table_function2_t = Option<
    unsafe extern "C" fn(
        ctx: duckdb_client_context,
        info: duckdb_function_info,
        output: duckdb_data_chunk,
    ),
>;
pub type duckdb_scalar_function_t = Option<
    unsafe extern "C" fn(
        args: duckdb_data_chunk,
        state: duckdb_expression_state,
        result: duckdb_vector,
    ),
>;

pub type FileOpenFlags = u8;

extern "C" {
    pub fn jfr_scan_create_view(
        context: duckdb_client_context,
        filename: *const c_char,
        tablename: *const c_char,
    );

    pub fn duckdb_create_struct_type(
        n_pairs: idx_t,
        names: *mut *const c_char,
        types: *const duckdb_logical_type,
    ) -> duckdb_logical_type;

    pub fn duckdb_create_table_function2() -> duckdb_table_function;

    pub fn duckdb_table_function2_set_function(
        table_function: duckdb_table_function,
        function: duckdb_table_function2_t,
    );

    pub fn duckdb_table_function2_set_bind(
        table_function: duckdb_table_function,
        bind: duckdb_table_function2_bind_t,
    );

    pub fn duckdb_table_function2_set_init(
        table_function: duckdb_table_function,
        bind: duckdb_table_function_init_t,
    );

    pub fn duckdb_register_table_function2(
        con: duckdb_connection,
        function: duckdb_table_function,
    ) -> duckdb_state;

    pub fn duckdb_function2_get_bind_data(info: duckdb_function_info) -> *mut c_void;

    pub fn duckdb_function2_get_init_data(info: duckdb_function_info) -> *mut c_void;

    pub fn duckdb_open_file(
        context: duckdb_client_context,
        path: *const c_char,
        flags: FileOpenFlags,
    ) -> duckdb_file_handle;

    pub fn duckdb_file_get_size(handle: duckdb_file_handle) -> i64;

    pub fn duckdb_file_read(handle: duckdb_file_handle, buffer: *mut c_void, nr_bytes: i64) -> i64;

    pub fn duckdb_file_seek(handle: duckdb_file_handle, pos: u64);

    pub fn duckdb_file_close(handle: duckdb_file_handle);

    pub fn duckdb_create_scalar_function(name: *const c_char, return_type: duckdb_logical_type) -> duckdb_scalar_function;

    pub fn duckdb_scalar_function_add_parameter(function: duckdb_scalar_function, ty: duckdb_logical_type);

    pub fn duckdb_scalar_function_set_function(scalar_function: duckdb_scalar_function, function: duckdb_scalar_function_t);

    pub fn duckdb_register_scalar_function(con: duckdb_connection, scalar_function: duckdb_scalar_function) -> duckdb_state;

    pub fn duckdb_destroy_scalar_function(function: *mut duckdb_scalar_function);

    pub fn duckdb_get_string(vector: duckdb_vector, index: idx_t) -> *const c_char;
}

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
