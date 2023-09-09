use crate::duckdb::bind_info::BindInfo;
use crate::duckdb::bindings::{
    duckdb_bind_info, duckdb_bind_set_error, duckdb_client_context, duckdb_data_chunk,
    duckdb_function_info, duckdb_function_set_error, duckdb_init_info, jfr_scan_create_view,
    LogicalTypeId,
};
use crate::duckdb::file::FileHandle;
use crate::duckdb::function_info::FunctionInfo;
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::malloc_struct;
use crate::duckdb::table_function::TableFunction;
use crate::Result;
use jfrs::reader::type_descriptor::TypeDescriptor;
use jfrs::reader::JfrReader;
use std::ffi::{c_char, CStr, CString};

pub fn build_table_function_def() -> Result<TableFunction> {
    let table_function = TableFunction::new();
    table_function.set_name("jfr_attach")?;
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.set_function(Some(jfr_attach_func));
    table_function.set_bind(Some(jfr_attach_bind));
    table_function.set_init(Some(jfr_attach_init));
    Ok(table_function)
}

unsafe extern "C" fn jfr_attach_bind(context: duckdb_client_context, info: duckdb_bind_info) {
    if let Err(err) = bind(context, info) {
        if let Ok(cstr) = CString::new(err.to_string()) {
            duckdb_bind_set_error(info, cstr.into_raw());
        }
    }
}

unsafe extern "C" fn jfr_attach_init(_info: duckdb_init_info) {
    // noop
}

unsafe fn bind(_context: duckdb_client_context, info: duckdb_bind_info) -> Result<()> {
    let info = BindInfo::from(info);

    let param0 = info.get_parameter(0);
    let filename = param0.get_varchar()?;

    let bind_data = malloc_struct::<AttachBindData>();
    (*bind_data).done = false;
    (*bind_data).filename = filename.as_ptr().cast();
    info.set_bind_data(bind_data.cast(), None);
    info.add_result_column("Success", &LogicalType::new(LogicalTypeId::Boolean))?;
    Ok(())
}

unsafe extern "C" fn jfr_attach_func(
    context: duckdb_client_context,
    info: duckdb_function_info,
    _output_raw: duckdb_data_chunk,
) {
    if let Err(err) = attach(context, info) {
        if let Ok(cstr) = CString::new(err.to_string()) {
            duckdb_function_set_error(info, cstr.into_raw());
        }
    }
}

unsafe fn attach(context: duckdb_client_context, info: duckdb_function_info) -> Result<()> {
    let info = FunctionInfo::from(info);
    let bind_data = info.get_bind_data::<AttachBindData>().as_mut().unwrap();

    if bind_data.done {
        return Ok(());
    }

    let filename = CStr::from_ptr(bind_data.filename);
    let filename_rs = filename.to_str()?;

    let mut reader = JfrReader::new(FileHandle::open(context, filename_rs));
    if let Some(chunk) = reader.chunks().next() {
        let (_, chunk) = chunk?;
        let mut types: Vec<&TypeDescriptor> = chunk.metadata.type_pool.get_types().collect();
        types.sort_by_key(|t| t.name());
        for tpe in types.iter() {
            match tpe.super_type() {
                // https://github.com/moditect/jfr-analytics/blob/352daa673e22e62c2bf4efe42b4d01b1d3c83d01/src/main/java/org/moditect/jfranalytics/JfrSchema.java#L71
                // https://github.com/adoptium/jdk11u/blob/jdk-11.0.21%2B6/src/jdk.jfr/share/classes/jdk/jfr/internal/MetadataReader.java#L223
                // https://github.com/adoptium/jdk11u/blob/jdk-11.0.21%2B6/src/jdk.jfr/share/classes/jdk/jfr/internal/MetadataReader.java#L260-L261
                Some("jdk.jfr.Event") if !tpe.fields.is_empty() => {
                    jfr_scan_create_view(
                        context,
                        filename.as_ptr(),
                        CString::new(tpe.name())?.as_ptr(),
                    );
                }
                _ => {}
            }
        }
    }
    bind_data.done = true;

    Ok(())
}

struct AttachBindData {
    done: bool,
    filename: *const c_char,
}
