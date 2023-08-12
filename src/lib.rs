use std::ffi::{c_char, c_void, CString};
use std::fmt::format;
use std::ptr::null_mut;
use duckdb_extension_framework::{check, Connection, Database, DataChunk, LogicalType, LogicalTypeId, malloc_struct};
use duckdb_extension_framework::duckly::{duckdb_bind_info, duckdb_connect, duckdb_connection, duckdb_data_chunk, duckdb_function_info, duckdb_init_info, duckdb_library_version};
use duckdb_extension_framework::table_functions::{BindInfo, FunctionInfo, InitInfo, TableFunction};

#[no_mangle]
pub unsafe extern "C" fn jfr_init_rust(db: *mut c_void) {
    init(db).expect("init failed");
    // let db = Database::from_cpp_duckdb(db);
    // let conn = db.connect().unwrap();
    // // let mut conn: duckdb_connection = null_mut();
    // // check!(duckdb_connect(db, &mut conn));
    // // let conn = Connection::from(conn);
    // // let conn = db.connect().unwrap();
    // // init(db).unwrap();
    // println!("init!!");
}

unsafe fn init(db: *mut c_void) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::from_cpp_duckdb(db);
    let conn = db.connect()?;
    let table_function = build_table_function_def();
    conn.register_table_function(table_function)?;
    jfr_create_view(
        conn.get_ptr().cast(),
        // CString::new("foo_file").unwrap().into_raw(),
        CString::new("hello_view").unwrap().into_raw(),
    );
    println!("init!!????");
    Ok(())
}

#[no_mangle]
pub extern "C" fn jfr_version_rust() -> *const c_char {
    unsafe { duckdb_library_version() }
}

extern "C" {
    fn jfr_create_view(conn: *mut c_void, tablename: *const c_char);
}

fn build_table_function_def() -> TableFunction {
    let table_function = TableFunction::new();
    table_function.set_name("jfr_scan");
    let logical_type = LogicalType::new(LogicalTypeId::Varchar);
    table_function.add_parameter(&logical_type);
    table_function.set_function(Some(jfr_scan_func));
    table_function.set_init(Some(jfr_scan_init));
    table_function.set_bind(Some(jfr_scan_bind));
    table_function
}

unsafe extern "C" fn jfr_scan_bind(info: duckdb_bind_info) {
    let info = BindInfo::from(info);
    info.add_result_column("mycolumn", LogicalType::new(LogicalTypeId::Varchar));
    let param = info.get_parameter(0);
    let bind_data = malloc_struct::<JfrBindData>();
    (*bind_data).name = param.get_varchar().into_raw();
    info.set_bind_data(bind_data.cast(), None);
}

unsafe extern "C" fn jfr_scan_init(info: duckdb_init_info) {
    let info = InitInfo::from(info);
    let init_data = malloc_struct::<JfrInitData>();
    (*init_data).done = false;
    info.set_init_data(init_data.cast(), None);
}

unsafe extern "C" fn jfr_scan_func(
    info: duckdb_function_info,
    output: duckdb_data_chunk
) {
    let info = FunctionInfo::from(info);
    let output = DataChunk::from(output);
    let init_data = info.get_init_data::<JfrInitData>();
    let bind_data = info.get_bind_data::<JfrBindData>();

    if (*init_data).done {
        output.set_size(0);
    } else {
        (*init_data).done = true;
        let vector = output.get_vector::<&str>(0);
        let name = CString::from_raw((*bind_data).name);
        let res = CString::new(format!("Hello {}", name.to_str().unwrap())).unwrap();
        (*bind_data).name = name.into_raw();
        vector.assign_string_element(0, res.into_raw());
        output.set_size(1);
    }
}

#[repr(C)]
struct JfrInitData {
    done: bool,
}

#[repr(C)]
struct JfrBindData {
    name: *mut c_char,
}
