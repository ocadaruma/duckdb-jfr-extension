use crate::duckdb::bind_info::BindInfo;
use crate::duckdb::bindings::{
    duckdb_bind_info, duckdb_client_context, duckdb_data_chunk, duckdb_function_info,
    duckdb_init_info, LogicalTypeId,
};
use crate::duckdb::file::FileHandle;
use crate::duckdb::function_info::FunctionInfo;
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::table_function::TableFunction;
use crate::duckdb::value::Value;
use crate::duckdb::view::TableFunctionView;
use crate::{jfr_reader, Result};
use anyhow::anyhow;
use jfrs::reader::type_descriptor::TypeDescriptor;

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
    let info = BindInfo::from_ptr(info);
    if let Err(err) = bind(context, &info) {
        info.set_error(&err);
    }
}

unsafe extern "C" fn jfr_attach_init(_info: duckdb_init_info) {
    // noop
}

unsafe fn bind(_context: duckdb_client_context, info: &BindInfo) -> Result<()> {
    let filename = info.get_parameter(0).get_varchar().as_str()?.to_string();

    info.set_bind_data(Box::new(AttachBindData {
        done: false,
        filename,
    }));
    info.add_result_column("Success", &LogicalType::new(LogicalTypeId::Boolean))?;
    Ok(())
}

unsafe extern "C" fn jfr_attach_func(
    context: duckdb_client_context,
    info: duckdb_function_info,
    _output_raw: duckdb_data_chunk,
) {
    let info = FunctionInfo::from_ptr(info);
    if let Err(err) = attach(context, &info) {
        info.set_error(&err);
    }
}

fn attach(context: duckdb_client_context, info: &FunctionInfo) -> Result<()> {
    let mut bind_data = info
        .get_bind_data::<AttachBindData>()
        .ok_or(anyhow!("bind_data is null"))?;
    if bind_data.done {
        return Ok(());
    }

    let filename = bind_data.filename.as_str();
    let mut reader = jfr_reader(filename, FileHandle::open(context, filename))?;
    if let Some(chunk) = reader.chunk_metadata().next() {
        let (_, chunk) = chunk?;
        let mut types: Vec<&TypeDescriptor> = chunk.metadata.type_pool.get_types().collect();
        types.sort_by_key(|t| t.name());
        for tpe in types.iter() {
            match tpe.super_type() {
                // https://github.com/moditect/jfr-analytics/blob/352daa673e22e62c2bf4efe42b4d01b1d3c83d01/src/main/java/org/moditect/jfranalytics/JfrSchema.java#L71
                // https://github.com/adoptium/jdk11u/blob/jdk-11.0.21%2B6/src/jdk.jfr/share/classes/jdk/jfr/internal/MetadataReader.java#L223
                // https://github.com/adoptium/jdk11u/blob/jdk-11.0.21%2B6/src/jdk.jfr/share/classes/jdk/jfr/internal/MetadataReader.java#L260-L261
                Some("jdk.jfr.Event") if !tpe.fields.is_empty() => {
                    let view = TableFunctionView::new();
                    view.set_name(tpe.name())?;
                    view.set_function_name("jfr_scan")?;
                    view.add_parameter(&Value::new_varchar(bind_data.filename.as_str())?);
                    view.add_parameter(&Value::new_varchar(tpe.name())?);
                    view.register(context)?;
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
    filename: String,
}
