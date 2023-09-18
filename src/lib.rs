mod duckdb;
mod jfr_attach;
mod jfr_scan;
mod jfr_schema;

use crate::duckdb::Database;

use crate::duckdb::bindings::{
    duckdb_data_chunk, duckdb_data_chunk_get_size, duckdb_data_chunk_get_vector, duckdb_get_string,
    duckdb_library_version, duckdb_list_entry, duckdb_list_vector_get_child,
    duckdb_scalar_function_info, duckdb_scalar_function_set_error, duckdb_struct_vector_get_child,
    duckdb_unified_vector_validity_row_is_valid, duckdb_validity_set_row_invalid, duckdb_vector,
    duckdb_vector_ensure_validity_writable, duckdb_vector_get_validity, duckdb_vector_is_constant,
    LogicalTypeId,
};
use crate::duckdb::function_info::ScalarFunctionInfo;
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::scalar_function::ScalarFunction;
use crate::duckdb::unified_vector::UnifiedVector;
use crate::duckdb::vector::Vector;
use regex::Regex;
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::{null_mut, slice_from_raw_parts};
use std::slice::from_raw_parts;

type Result<T> = anyhow::Result<T>;

// TODO:
// - multi chunk
// - CString-pool fails with non-constant-pool-String
// - error handlings
// - interval support
// - projection pushdown
// - cleanup comments
// - malloc/free
// - assign_string_element_len without memcpy?
// - null handling in stacktrace_matches

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
    f.set_name("stacktrace_matches")?;
    f.add_parameter(&stacktrace_type()?);
    f.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    f.set_return_type(&LogicalType::new(LogicalTypeId::Boolean));
    f.set_function(Some(stacktrace_matches_function));
    Ok(f)
}

// struct InitData {
//     regex: Regex,
// }

unsafe extern "C" fn stacktrace_matches_function(
    info: duckdb_scalar_function_info,
    args: duckdb_data_chunk,
    result: duckdb_vector,
) {
    if let Err(err) = stacktrace_matches(info, args, result) {
        if let Ok(cstr) = CString::new(err.to_string()) {
            duckdb_scalar_function_set_error(info, cstr.into_raw());
        }
    }
}

unsafe fn stacktrace_matches(
    _info: duckdb_scalar_function_info,
    args: duckdb_data_chunk,
    result: duckdb_vector,
) -> Result<()> {
    let count = duckdb_data_chunk_get_size(args);
    if count == 0 {
        return Ok(());
    }

    let result_vector = Vector::from(result);
    let frames = Vector::from(duckdb_data_chunk_get_vector(args, 0)).get_struct_child(1);
    let method_vector = frames
        .get_list_child()
        .get_struct_child(0) // method
        .get_struct_child(1) // name
        .get_struct_child(0); // string
    let type_vector = frames
        .get_list_child()
        .get_struct_child(0) // method
        .get_struct_child(0) // type
        .get_struct_child(1) // name
        .get_struct_child(0); // string
    let unified_method = UnifiedVector::new(method_vector.ptr(), count);
    let unified_type = UnifiedVector::new(type_vector.ptr(), count);

    let patterns_vec = Vector::from(duckdb_data_chunk_get_vector(args, 1));
    let patterns = UnifiedVector::new(patterns_vec.ptr(), count);

    let constant_pattern = if patterns_vec.is_constant() {
        // count is non-zero here
        let p = patterns.get_string(0);
        let p = from_raw_parts(p.data.cast::<u8>(), p.size as usize);
        let p = std::str::from_utf8(p)?;
        Some(Regex::new(p)?)
    } else {
        None
    };

    for i in 0..count {
        let entry = frames
            .get_data::<duckdb_list_entry>()
            .add(i as usize)
            .read();
        let mut matched = false;

        let adhoc_pattern = if constant_pattern.is_none() {
            let p = patterns.get_string(i);
            let p = from_raw_parts(p.data.cast::<u8>(), p.size as usize);
            let p = std::str::from_utf8(p)?;
            Some(Regex::new(p)?)
        } else {
            None
        };

        let pattern = if let Some(r) = &constant_pattern {
            r
        } else {
            adhoc_pattern.as_ref().unwrap()
        };

        for j in 0..entry.length {
            let type_name = unified_type.get_string(entry.offset + j);
            let type_name = from_raw_parts(type_name.data.cast::<u8>(), type_name.size as usize);
            let type_name = std::str::from_utf8(type_name)?;

            let method_name = unified_method.get_string(entry.offset + j);
            let method_name =
                from_raw_parts(method_name.data.cast::<u8>(), method_name.size as usize);
            let method_name = std::str::from_utf8(method_name)?;

            let s = format!("{}.{}", type_name, method_name);
            if pattern.is_match(s.as_str()) {
                matched = true;
                break;
            }
        }

        result_vector
            .get_data::<bool>()
            .add(i as usize)
            .write(matched);
    }
    Ok(())
}

fn stacktrace_type() -> Result<LogicalType> {
    LogicalType::new_struct_type(&[
        ("truncated", LogicalType::new(LogicalTypeId::Boolean)),
        (
            "frames",
            LogicalType::new_list_type(&LogicalType::new_struct_type(&[
                (
                    "method",
                    LogicalType::new_struct_type(&[
                        (
                            "type",
                            LogicalType::new_struct_type(&[
                                (
                                    "classLoader",
                                    LogicalType::new_struct_type(&[(
                                        "name",
                                        LogicalType::new_struct_type(&[(
                                            "string",
                                            LogicalType::new(LogicalTypeId::Varchar),
                                        )])?,
                                    )])?,
                                ),
                                (
                                    "name",
                                    LogicalType::new_struct_type(&[(
                                        "string",
                                        LogicalType::new(LogicalTypeId::Varchar),
                                    )])?,
                                ),
                                (
                                    "package",
                                    LogicalType::new_struct_type(&[(
                                        "name",
                                        LogicalType::new_struct_type(&[(
                                            "string",
                                            LogicalType::new(LogicalTypeId::Varchar),
                                        )])?,
                                    )])?,
                                ),
                                ("modifiers", LogicalType::new(LogicalTypeId::Integer)),
                            ])?,
                        ),
                        (
                            "name",
                            LogicalType::new_struct_type(&[(
                                "string",
                                LogicalType::new(LogicalTypeId::Varchar),
                            )])?,
                        ),
                        (
                            "descriptor",
                            LogicalType::new_struct_type(&[(
                                "string",
                                LogicalType::new(LogicalTypeId::Varchar),
                            )])?,
                        ),
                        ("modifiers", LogicalType::new(LogicalTypeId::Integer)),
                        ("hidden", LogicalType::new(LogicalTypeId::Boolean)),
                    ])?,
                ),
                ("lineNumber", LogicalType::new(LogicalTypeId::Integer)),
                ("bytecodeIndex", LogicalType::new(LogicalTypeId::Integer)),
                (
                    "type",
                    LogicalType::new_struct_type(&[(
                        "description",
                        LogicalType::new(LogicalTypeId::Varchar),
                    )])?,
                ),
            ])?),
        ),
    ])
}

#[no_mangle]
pub extern "C" fn libduckdb_jfr_extension_version() -> *const c_char {
    unsafe { duckdb_library_version() }
}
