mod duckdb;
mod jfr_schema;
mod jfr_scan;
mod jfr_attach;

use crate::duckdb::Database;
use libduckdb_sys::{
    duckdb_library_version
};

use std::ffi::{c_char, c_void};

type Result<T> = anyhow::Result<T>;

// TODO:
// - multi chunk
// - CString-pool fails with non-constant-pool-String
// - error handlings
// - interval support
// - projection pushdown
// - cleanup comments

#[no_mangle]
pub unsafe extern "C" fn libduckdb_jfr_extension_init(db: *mut c_void) {
    let res = init(db);
    if let Err(err) = res {
        println!("Error: {}", err);
    }
}

unsafe fn init(db: *mut c_void) -> Result<()> {
    let db = Database::from(db);
    let conn = db.connect()?;
    conn.register_table_function(&jfr_scan::build_table_function_def())?;
    conn.register_table_function(&jfr_attach::build_table_function_def())?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn libduckdb_jfr_extension_version() -> *const c_char {
    unsafe { duckdb_library_version() }
}
