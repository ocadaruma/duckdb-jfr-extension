mod duckdb;
mod jfr_attach;
mod jfr_scan;
mod jfr_schema;

use crate::duckdb::Database;

use crate::duckdb::bindings::{duckdb_database, duckdb_library_version};

use crate::duckdb::file::FileHandle;
use flate2::read::GzDecoder;
use jfrs::reader::JfrReader;
use log::error;
use std::ffi::c_char;
use std::io::{Cursor, Read};

type Result<T> = anyhow::Result<T>;

// TODO:
// - interval support
// - cleanup comments
// - add tests
// - wrap all raw C API calls (to prevent memory leaks / unsafes)

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn libduckdb_jfr_extension_init(db: duckdb_database) {
    let _ = env_logger::try_init();

    let res = init(db);
    if let Err(err) = res {
        error!("init error: {:?}", err);
    }
}

unsafe fn init(db: duckdb_database) -> Result<()> {
    let db = Database::from_ptr(db);
    let conn = db.connect()?;
    conn.register_table_function(&jfr_scan::build_table_function_def()?)?;
    conn.register_table_function(&jfr_attach::build_table_function_def()?)?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn libduckdb_jfr_extension_version() -> *const c_char {
    unsafe { duckdb_library_version() }
}

fn jfr_reader(filename: &str, mut handle: FileHandle) -> Result<JfrReader<Cursor<Vec<u8>>>> {
    let mut buf = vec![];
    if filename.ends_with(".gz") {
        GzDecoder::new(handle).read_to_end(&mut buf)?;
        Ok(JfrReader::new(Cursor::new(buf)))
    } else {
        handle.read_to_end(&mut buf)?;
        Ok(JfrReader::new(Cursor::new(buf)))
    }
}
