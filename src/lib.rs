mod duckdb;
mod jfr_attach;
mod jfr_scan;
mod jfr_schema;

use crate::duckdb::Database;

use crate::duckdb::bindings::{duckdb_data_chunk, duckdb_data_chunk_get_size, duckdb_data_chunk_get_vector, duckdb_get_string, duckdb_library_version, duckdb_list_entry, duckdb_list_vector_get_child, duckdb_scalar_bind_info, duckdb_scalar_function_info, duckdb_scalar_init_info, duckdb_struct_vector_get_child, duckdb_vector, LogicalTypeId};
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::scalar_function::ScalarFunction;
use crate::duckdb::vector::Vector;
use regex::Regex;
use std::ffi::{c_char, c_void, CStr};
use std::ptr::slice_from_raw_parts;

type Result<T> = anyhow::Result<T>;

// TODO:
// - multi chunk
// - CString-pool fails with non-constant-pool-String
// - error handlings
// - interval support
// - projection pushdown
// - cleanup comments
// - malloc/free

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
    conn.register_table_function(&jfr_scan::build_table_function_def()?)?;
    conn.register_table_function(&jfr_attach::build_table_function_def()?)?;
    conn.register_scalar_function(&stacktrace_match_def()?)?;
    // conn.register_scalar_function(&jfr_stacktrace_match_def()?)?;
    // jfr_register_stacktrace_matches_function(conn.ptr());
    Ok(())
}

fn stacktrace_match_def() -> Result<ScalarFunction> {
    let f = ScalarFunction::new();
    f.set_name("stacktrace_match")?;
    f.set_return_type(&LogicalType::new(LogicalTypeId::Boolean));
    f.set_bind(Some(stacktrace_match_bind));
    f.set_init(Some(stacktrace_match_init));
    f.set_function(Some(stacktrace_match_function));
    Ok(f)
}

unsafe extern "C" fn stacktrace_match_bind(_info: duckdb_scalar_bind_info) {
    // noop
}

unsafe extern "C" fn stacktrace_match_init(_info: duckdb_scalar_init_info) {
    // noop
}

unsafe extern "C" fn stacktrace_match_function(
    _info: duckdb_scalar_function_info,
    args: duckdb_data_chunk,
    result: duckdb_vector
) {
    let count = duckdb_data_chunk_get_size(args);
    let vector = Vector::from(result);
    for i in 0..count {
        vector.get_data::<bool>().add(i as usize).write(i % 2 == 0);
    }
}

// fn jfr_stacktrace_match_def() -> Result<ScalarFunction> {
//     let f = ScalarFunction::new(
//         "stacktrace_matches",
//         &LogicalType::new(LogicalTypeId::Boolean),
//     )?;
//     // f.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
//     f.add_parameter(&stacktrace_type()?);
//     f.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
//     f.set_function(Some(stacktrace_matches));
//     Ok(f)
// }
//
// fn stacktrace_type() -> Result<LogicalType> {
//     LogicalType::new_struct_type(&[
//         ("truncated", LogicalType::new(LogicalTypeId::Boolean)),
//         (
//             "frames",
//             LogicalType::new_list_type(&LogicalType::new_struct_type(&[
//                 (
//                     "method",
//                     LogicalType::new_struct_type(&[
//                         (
//                             "type",
//                             LogicalType::new_struct_type(&[
//                                 (
//                                     "classLoader",
//                                     LogicalType::new_struct_type(&[(
//                                         "name",
//                                         LogicalType::new_struct_type(&[(
//                                             "string",
//                                             LogicalType::new(LogicalTypeId::Varchar),
//                                         )])?,
//                                     )])?,
//                                 ),
//                                 (
//                                     "name",
//                                     LogicalType::new_struct_type(&[(
//                                         "string",
//                                         LogicalType::new(LogicalTypeId::Varchar),
//                                     )])?,
//                                 ),
//                                 (
//                                     "package",
//                                     LogicalType::new_struct_type(&[(
//                                         "name",
//                                         LogicalType::new_struct_type(&[(
//                                             "string",
//                                             LogicalType::new(LogicalTypeId::Varchar),
//                                         )])?,
//                                     )])?,
//                                 ),
//                                 ("modifiers", LogicalType::new(LogicalTypeId::Integer)),
//                             ])?,
//                         ),
//                         (
//                             "name",
//                             LogicalType::new_struct_type(&[(
//                                 "string",
//                                 LogicalType::new(LogicalTypeId::Varchar),
//                             )])?,
//                         ),
//                         (
//                             "descriptor",
//                             LogicalType::new_struct_type(&[(
//                                 "string",
//                                 LogicalType::new(LogicalTypeId::Varchar),
//                             )])?,
//                         ),
//                         ("modifiers", LogicalType::new(LogicalTypeId::Integer)),
//                         ("hidden", LogicalType::new(LogicalTypeId::Boolean)),
//                     ])?,
//                 ),
//                 ("lineNumber", LogicalType::new(LogicalTypeId::Integer)),
//                 ("bytecodeIndex", LogicalType::new(LogicalTypeId::Integer)),
//                 (
//                     "type",
//                     LogicalType::new_struct_type(&[(
//                         "description",
//                         LogicalType::new(LogicalTypeId::Varchar),
//                     )])?,
//                 ),
//             ])?),
//         ),
//     ])
// }
//
// // unsafe extern "C" fn jfr_stacktrace_match(
// //     args: duckdb_data_chunk,
// //     state: duckdb_expression_state,
// //     result: duckdb_vector,
// // ) {
// //     let count = duckdb_data_chunk_get_size(args);
// //     let vector = Vector::from(duckdb_data_chunk_get_vector(args, 0));
// //     let result_vector = Vector::from(result);
// //     for i in 0..count {
// //         let str = CStr::from_ptr(duckdb_get_string(vector.ptr(), i)).to_str().expect("invalid utf8");
// //         result_vector.get_data::<bool>().offset(i as isize).write(str.contains("foo"));
// //     }
// // }
//
// unsafe extern "C" fn stacktrace_matches(
//     args: duckdb_data_chunk,
//     _state: duckdb_expression_state,
//     result: duckdb_vector,
// ) {
//     let stacktrace_vector = duckdb_data_chunk_get_vector(args, 0);
//     let pattern_vector = duckdb_data_chunk_get_vector(args, 1);
//
//     let count = duckdb_data_chunk_get_size(args);
//     let frames = Vector::from(duckdb_struct_vector_get_child(stacktrace_vector, 1));
//     let frame = duckdb_list_vector_get_child(frames.ptr());
//     let method = duckdb_struct_vector_get_child(frame, 0);
//     let name = duckdb_struct_vector_get_child(method, 1);
//     let string = Vector::from(duckdb_struct_vector_get_child(name, 0));
//
//     let result_vector = Vector::from(result);
//     for i in 0..count {
//         duckdb_get_string(pattern_vector, i);
//         // let pattern = CStr::from_ptr(duckdb_get_string(pattern_vector, i)).to_str().expect("invalid utf8");
//         let _reg = Regex::new(".*fsync.*").expect("invalid regex");
//
//         let entry = frames
//             .get_data::<duckdb_list_entry>()
//             .offset(i as isize)
//             .read();
//         let result = false;
//         // println!("entry.offset: {}, length: {}", entry.offset, entry.length);
//         for j in 0..entry.length {
//             // duckdb_get_string(string.ptr(), entry.offset + j);
//             let _str = CStr::from_ptr(duckdb_get_string(string.ptr(), entry.offset + j))
//                 .to_str()
//                 .expect("invalid utf8");
//             // if reg.is_match(str) {
//             //     result = true;
//             //     break;
//             // }
//         }
//         slice_from_raw_parts()
//         result_vector
//             .get_data::<bool>()
//             .offset(i as isize)
//             .write(result);
//     }
// }

#[no_mangle]
pub extern "C" fn libduckdb_jfr_extension_version() -> *const c_char {
    unsafe { duckdb_library_version() }
}
