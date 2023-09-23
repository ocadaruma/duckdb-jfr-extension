mod duckdb;
mod jfr_attach;
mod jfr_scan;
mod jfr_schema;
mod stacktrace_matches;

use crate::duckdb::Database;

use crate::duckdb::bindings::{duckdb_database, duckdb_library_version};

use std::ffi::c_char;

type Result<T> = anyhow::Result<T>;

// TODO:
// - error handling
// - interval support
// - projection pushdown
// - cleanup comments
// - null handling in stacktrace_matches
// - wrap all raw C API calls (to prevent memory leaks / unsafes)
// - performance optimization by dictionary vectors

#[no_mangle]
pub unsafe extern "C" fn libduckdb_jfr_extension_init(db: duckdb_database) {
    let res = init(db);
    if let Err(err) = res {
        println!("Error: {}", err);
    }
}

unsafe fn init(db: duckdb_database) -> Result<()> {
    let db = Database::from_ptr(db);
    let conn = db.connect()?;
    conn.register_table_function(&jfr_scan::build_table_function_def()?)?;
    conn.register_table_function(&jfr_attach::build_table_function_def()?)?;
    conn.register_scalar_function(&stacktrace_matches::stacktrace_matches_def()?)?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn libduckdb_jfr_extension_version() -> *const c_char {
    unsafe { duckdb_library_version() }
}
