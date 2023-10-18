use crate::duckdb::bind_info::BindInfo;
use crate::duckdb::bindings::{
    duckdb_bind_info, duckdb_client_context, duckdb_data_chunk, duckdb_data_chunk_get_vector,
    duckdb_function_info, duckdb_init_info, duckdb_list_entry, duckdb_list_vector_get_child,
    duckdb_list_vector_reserve, duckdb_list_vector_set_size, duckdb_struct_vector_get_child,
    duckdb_validity_set_row_invalid, duckdb_vector, duckdb_vector_ensure_validity_writable,
    duckdb_vector_get_validity, duckdb_vector_size, idx_t, LogicalTypeId,
};
use crate::duckdb::data_chunk::DataChunk;
use crate::duckdb::file::FileHandle;
use crate::duckdb::function_info::FunctionInfo;
use crate::duckdb::init_info::InitInfo;
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::table_function::TableFunction;
use crate::duckdb::vector::Vector;
use crate::jfr_schema::JfrField;
use crate::Result;
use jfrs::reader::event::Accessor;
use jfrs::reader::type_descriptor::{TickUnit, Unit};
use jfrs::reader::value_descriptor::{Primitive, ValueDescriptor};
use jfrs::reader::{Chunk, ChunkReader, JfrReader};
use std::collections::hash_map::Entry;
use std::ffi::CString;

use anyhow::anyhow;
use rustc_hash::FxHashMap;

const STACK_TRACE_TYPE: &str = "jdk.types.StackTrace";
const ROW_ID_COLUMN_ID: usize = usize::MAX;

pub fn build_table_function_def() -> Result<TableFunction> {
    let table_function = TableFunction::new();
    table_function.set_name("jfr_scan")?;
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.set_function(Some(jfr_scan_func));
    table_function.set_init(Some(jfr_scan_init));
    table_function.set_bind(Some(jfr_scan_bind));
    table_function.set_supports_projection_pushdown(true);
    Ok(table_function)
}

unsafe extern "C" fn jfr_scan_bind(context: duckdb_client_context, info: duckdb_bind_info) {
    let info = BindInfo::from_ptr(info);
    if let Err(err) = bind(context, &info) {
        info.set_error(&err);
    }
}

unsafe fn bind(context: duckdb_client_context, info: &BindInfo) -> Result<()> {
    let filename = info.get_parameter(0).get_varchar().as_str()?.to_string();
    let table_name = info.get_parameter(1).get_varchar().as_str()?.to_string();

    let mut reader = JfrReader::new(FileHandle::open(context, filename.as_str()));
    let (_, chunk) = reader
        .chunk_metadata()
        .next()
        .ok_or(anyhow!("no chunk"))??;

    let (root, count) = JfrField::from_chunk(&chunk, table_name.as_str())?;
    for s in root.children.iter() {
        info.add_result_column(s.name.as_str(), &create_type(s)?)?;
    }

    info.set_bind_data(Box::new(ScanBindData {
        filename,
        table_name,
        schema: root,
        field_count: count,
    }));
    Ok(())
}

fn create_type(field: &JfrField) -> Result<LogicalType> {
    match map_primitive_type(field) {
        Some(t) => Ok(t),
        None => {
            let mut shape = vec![];
            for s in field.children.iter() {
                if !s.valid {
                    continue;
                }
                let t = create_type(s)?;
                let t = if s.is_array {
                    LogicalType::new_list_type(&t)
                } else {
                    t
                };
                shape.push((s.name.as_str(), t));
            }
            LogicalType::new_struct_type(shape.as_slice())
        }
    }
}

fn map_primitive_type(field: &JfrField) -> Option<LogicalType> {
    match field.type_name.as_str() {
        "int" => Some(LogicalType::new(LogicalTypeId::Integer)),
        "long" => match (field.tick_unit, field.unit) {
            (Some(TickUnit::Timestamp), _)
            | (_, Some(Unit::EpochNano | Unit::EpochMilli | Unit::EpochSecond)) => {
                Some(LogicalType::new(LogicalTypeId::Timestamp))
            }
            _ => Some(LogicalType::new(LogicalTypeId::Bigint)),
        },
        "float" => Some(LogicalType::new(LogicalTypeId::Float)),
        "double" => Some(LogicalType::new(LogicalTypeId::Double)),
        "char" => Some(LogicalType::new(LogicalTypeId::Varchar)),
        "boolean" => Some(LogicalType::new(LogicalTypeId::Boolean)),
        "short" => Some(LogicalType::new(LogicalTypeId::Smallint)),
        "byte" => Some(LogicalType::new(LogicalTypeId::Tinyint)),
        "java.lang.String" => Some(LogicalType::new(LogicalTypeId::Varchar)),
        // For performance reasons, we store stacktrace as varchar of concatenated frames
        // instead of parsing it into a list of stack frame struct.
        STACK_TRACE_TYPE => Some(LogicalType::new(LogicalTypeId::Varchar)),
        _ => None,
    }
}

unsafe extern "C" fn jfr_scan_init(info: duckdb_init_info) {
    let info = InitInfo::from_ptr(info);
    let mut projection = vec![];
    for i in 0..info.projected_column_count() {
        projection.push(info.column_index(i));
    }
    info.set_init_data(Box::new(ScanInitData {
        done: false,
        chunks: vec![],
        chunk_idx: -1,
        offset_in_chunk: 0,
        projection,
    }));
}

unsafe extern "C" fn jfr_scan_func(
    context: duckdb_client_context,
    info: duckdb_function_info,
    output_raw: duckdb_data_chunk,
) {
    let info = FunctionInfo::from_ptr(info);
    if let Err(err) = scan(context, &info, output_raw) {
        info.set_error(&err);
    }
}

unsafe fn scan(
    context: duckdb_client_context,
    info: &FunctionInfo,
    output_raw: duckdb_data_chunk,
) -> Result<()> {
    let output = DataChunk::from_ptr(output_raw);
    let mut init_data = info
        .get_init_data::<ScanInitData>()
        .ok_or(anyhow!("init_data is null"))?;
    let bind_data = info
        .get_bind_data::<ScanBindData>()
        .ok_or(anyhow!("bind_data is null"))?;
    let filename = bind_data.filename.as_str();
    let table_name = bind_data.table_name.as_str();

    if init_data.done {
        output.set_size(0);
        return Ok(());
    }

    let schema = &bind_data.schema;

    // Read all data into memory first
    if init_data.chunk_idx < 0 {
        let mut reader = JfrReader::new(FileHandle::open(context, filename));
        for chunk in reader.chunks() {
            let (r, c) = chunk?;
            init_data.chunks.push((r, c));
        }
        if init_data.chunks.is_empty() {
            init_data.done = true;
            output.set_size(0);
            return Ok(());
        }
        init_data.chunk_idx = 0;
    }

    let vector_size = duckdb_vector_size() as usize;
    let mut pool = Pool::new(bind_data.field_count);

    let current_chunk_idx = init_data.chunk_idx;
    let mut row = 0;

    for chunk_idx in current_chunk_idx..(init_data.chunks.len() as isize) {
        let (mut reader, chunk) = init_data.chunks.swap_remove(chunk_idx as usize);
        for event in reader.events_from_offset(&chunk, init_data.offset_in_chunk) {
            let event = event?;
            if event.class.name() != table_name {
                continue;
            }
            if row >= vector_size {
                init_data.chunk_idx = chunk_idx;
                init_data.offset_in_chunk = event.byte_offset;
                init_data.chunks.insert(chunk_idx as usize, (reader, chunk));
                output.set_size(row);
                return Ok(());
            }

            for p in 0..init_data.projection.len() {
                let i = init_data.projection[p];
                if i == ROW_ID_COLUMN_ID {
                    assign(
                        duckdb_data_chunk_get_vector(output_raw, p as idx_t),
                        row,
                        42,
                    );
                    continue;
                }

                let field = &schema.children[i];
                if let Some(accessor) = event.value().get_field_raw(field.name.as_str()) {
                    populate_column(
                        row,
                        duckdb_data_chunk_get_vector(output_raw, p as idx_t),
                        &output,
                        &chunk,
                        chunk_idx as usize,
                        field,
                        accessor,
                        &mut pool,
                    )?;
                } else {
                    set_null(
                        duckdb_data_chunk_get_vector(output_raw, p as idx_t),
                        row,
                        field,
                        &mut pool,
                    );
                }
            }
            row += 1;
        }
        init_data.offset_in_chunk = 0;
        init_data.chunks.insert(chunk_idx as usize, (reader, chunk));
    }

    init_data.done = true;
    output.set_size(row);
    Ok(())
}

unsafe fn set_null(vector: duckdb_vector, row_idx: usize, field: &JfrField, pool: &mut Pool) {
    duckdb_vector_ensure_validity_writable(vector);
    duckdb_validity_set_row_invalid(duckdb_vector_get_validity(vector), row_idx as u64);

    // Struct children have their own validity bitmaps.
    // Hence we also need to set validity mask for them.
    // Struct children's validity is an AND of parent validity and the child's validity.
    // refs: https://arrow.apache.org/docs/12.0/format/Columnar.html#struct-validity
    if !field.children.is_empty() && !field.is_array {
        for (i, s) in field.children.iter().filter(|s| s.valid).enumerate() {
            let child_vector = pool.get_struct_child_vector(s.idx, vector, i as idx_t);
            set_null(child_vector, row_idx, s, pool);
        }
    }
}

unsafe fn populate_column(
    row_idx: usize,
    vector: duckdb_vector,
    output: &DataChunk,
    chunk: &Chunk,
    chunk_idx: usize,
    field: &JfrField,
    accessor: Accessor<'_>,
    pool: &mut Pool,
) -> Result<()> {
    // special handling for stack trace to parse it as a string
    // rather than a list of stack frame struct for performance reason
    if field.type_name == STACK_TRACE_TYPE {
        match accessor.value {
            ValueDescriptor::ConstantPool {
                class_id: _,
                constant_index,
            } => {
                let (stacktrace, len) =
                    pool.get_stacktrace(chunk_idx, *constant_index, &accessor)?;
                Vector::from_ptr(vector).assign_string_element_len(
                    row_idx,
                    stacktrace.as_ptr(),
                    *len,
                );
            }
            _ => {
                let (stacktrace, len) = Pool::parse_stacktrace(&accessor)?;
                Vector::from_ptr(vector).assign_string_element_len(
                    row_idx,
                    stacktrace.as_ptr(),
                    len,
                );
            }
        }
        return Ok(());
    }
    match accessor.resolve().map(|a| a.value) {
        Some(ValueDescriptor::Primitive(p)) => match p {
            Primitive::Integer(v) => assign(vector, row_idx, *v),
            Primitive::Long(v) => {
                let v = match (field.tick_unit, field.unit) {
                    (Some(TickUnit::Timestamp), _) => {
                        let ticks_per_nanos =
                            (chunk.header.ticks_per_second as f64) / 1_000_000_000.0;
                        (chunk.header.start_time_nanos
                            + ((*v - chunk.header.start_ticks) as f64 / ticks_per_nanos) as i64)
                            / 1_000
                    }
                    (_, Some(Unit::EpochNano)) => *v / 1_000,
                    (_, Some(Unit::EpochMilli)) => *v * 1_000,
                    (_, Some(Unit::EpochSecond)) => *v * 1_000_000,
                    _ => *v,
                };
                assign(vector, row_idx, v)
            }
            Primitive::Float(v) => assign(vector, row_idx, *v),
            Primitive::Double(v) => assign(vector, row_idx, *v),
            Primitive::Character(v) => {
                Vector::from_ptr(vector).assign_string_element_len(
                    row_idx,
                    v.string.as_ptr(),
                    v.len,
                );
            }
            Primitive::Boolean(v) => assign(vector, row_idx, *v),
            Primitive::Short(v) => assign(vector, row_idx, *v),
            Primitive::Byte(v) => assign(vector, row_idx, *v),
            Primitive::NullString => set_null(vector, row_idx, field, pool),
            Primitive::String(s) => {
                Vector::from_ptr(vector).assign_string_element_len(
                    row_idx,
                    s.string.as_ptr(),
                    s.len,
                );
            }
        },
        Some(ValueDescriptor::Object(obj)) => {
            for (ii, (i, s)) in field
                .children
                .iter()
                .enumerate()
                .map(|(i, s)| (i, s))
                .filter(|(_, s)| s.valid)
                .enumerate()
            {
                let child_vector = pool.get_struct_child_vector(s.idx, vector, ii as idx_t);
                populate_column(
                    row_idx,
                    child_vector,
                    output,
                    chunk,
                    chunk_idx,
                    s,
                    Accessor::new(chunk, &obj.fields[i]),
                    pool,
                )?;
            }
        }
        Some(ValueDescriptor::Array(arr)) => {
            let child_vector = duckdb_list_vector_get_child(vector);
            let child_offset = pool.get_list_child_row_idx(field.idx);
            duckdb_list_vector_reserve(vector, (child_offset + arr.len()) as u64);
            for (i, v) in arr.iter().enumerate() {
                populate_column(
                    child_offset + i,
                    child_vector,
                    output,
                    chunk,
                    chunk_idx,
                    field,
                    Accessor::new(chunk, v),
                    pool,
                )?;
            }
            Vector::from_ptr(vector)
                .get_data::<duckdb_list_entry>()
                .add(row_idx)
                .write(duckdb_list_entry {
                    length: arr.len() as u64,
                    offset: child_offset as u64,
                });
            pool.set_list_child_row_idx(field.idx, child_offset + arr.len());
            duckdb_list_vector_set_size(vector, (child_offset + arr.len()) as u64);
        }
        _ => {
            set_null(vector, row_idx, field, pool);
        }
    }
    Ok(())
}

unsafe fn assign<T>(vector: duckdb_vector, row_idx: usize, value: T) {
    let vector = Vector::from_ptr(vector);
    vector.get_data::<T>().add(row_idx).write(value);
}

struct ScanInitData {
    done: bool,
    chunks: Vec<(ChunkReader, Chunk)>,
    chunk_idx: isize,
    offset_in_chunk: u64,
    projection: Vec<usize>,
}

struct ScanBindData {
    filename: String,
    table_name: String,
    schema: JfrField,
    field_count: usize,
}

struct Pool {
    vectors: Vec<Option<duckdb_vector>>,
    list_children_idx: Vec<usize>,
    stacktrace_cache: FxHashMap<(usize, i64), (CString, usize)>,
}

impl Pool {
    fn new(field_count: usize) -> Self {
        Self {
            vectors: vec![None; field_count],
            list_children_idx: vec![0; field_count],
            stacktrace_cache: FxHashMap::default(),
        }
    }

    fn get_struct_child_vector(
        &mut self,
        field_idx: usize,
        parent: duckdb_vector,
        child_idx: idx_t,
    ) -> duckdb_vector {
        if self.vectors[field_idx].is_none() {
            self.vectors[field_idx] =
                Some(unsafe { duckdb_struct_vector_get_child(parent, child_idx) });
        }
        self.vectors[field_idx].unwrap()
    }

    fn get_list_child_row_idx(&self, field_idx: usize) -> usize {
        self.list_children_idx[field_idx]
    }

    fn set_list_child_row_idx(&mut self, field_idx: usize, row_idx: usize) {
        self.list_children_idx[field_idx] = row_idx;
    }

    fn get_stacktrace(
        &mut self,
        chunk_idx: usize,
        constant_idx: i64,
        accessor: &Accessor<'_>,
    ) -> Result<&(CString, usize)> {
        match self.stacktrace_cache.entry((chunk_idx, constant_idx)) {
            Entry::Occupied(o) => Ok(o.into_mut()),
            Entry::Vacant(v) => Ok(v.insert(Self::parse_stacktrace(accessor)?)),
        }
    }

    fn parse_stacktrace(accessor: &Accessor<'_>) -> Result<(CString, usize)> {
        let mut frames = vec![];
        for frame in accessor
            .get_field("frames")
            .and_then(|f| f.as_iter())
            .ok_or_else(|| anyhow!("failed to get frames"))?
        {
            let type_name = frame
                .get_field("method")
                .and_then(|m| m.get_field("type"))
                .and_then(|t| t.get_field("name"))
                .and_then(|n| n.get_field("string"))
                .and_then(|s| <&str>::try_from(s.value).ok())
                .ok_or_else(|| anyhow!("failed to get type name"))?
                .to_string();
            let method_name = frame
                .get_field("method")
                .and_then(|t| t.get_field("name"))
                .and_then(|n| n.get_field("string"))
                .and_then(|s| <&str>::try_from(s.value).ok())
                .ok_or_else(|| anyhow!("failed to get method name"))?
                .to_string();
            let frame_type = frame
                .get_field("type")
                .and_then(|t| t.get_field("description"))
                .and_then(|s| <&str>::try_from(s.value).ok())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("failed to get frame type"))?;
            let line_number = frame
                .get_field("lineNumber")
                .and_then(|l| <i32>::try_from(l.value).ok())
                .ok_or_else(|| anyhow!("failed to get line number"))?;
            let frame_string = match frame_type.as_str() {
                "Interpreted" | "JIT compiled" | "Inlined" | "C1 compiled"
                    if !type_name.is_empty() =>
                {
                    let line_num = if line_number > 0 {
                        format!(":{}", line_number)
                    } else {
                        "".to_string()
                    };
                    format!("{}.{}{}", type_name, method_name, line_num)
                }
                _ => method_name,
            };
            frames.push(frame_string);
        }
        let result = frames.join("\n");
        let len = result.len();
        Ok((CString::new(result)?, len))
    }
}

#[cfg(test)]
mod tests {
    use crate::duckdb::Database;
    use crate::jfr_scan::{build_table_function_def, Pool};
    use crate::Result;
    use duckdb::types::FromSql;
    use duckdb::Connection;
    use jfrs::reader::JfrReader;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_count() {
        let jfr =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/async-profiler-wall.jfr");
        let result = query_single(
            format!(
                "select count(*) from jfr_scan('{}', 'jdk.ExecutionSample')",
                jfr.to_str().unwrap()
            )
            .as_str(),
            0,
        )
        .unwrap();
        assert_eq!(428, result);
    }

    #[test]
    fn test_multiple_chunks() {
        let jfr =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/profiler-multichunk.jfr");
        let result = query_single(
            format!(
                "select count(*) from jfr_scan('{}', 'jdk.ExecutionSample')",
                jfr.to_str().unwrap()
            )
            .as_str(),
            0,
        )
        .unwrap();
        assert_eq!(8888, result);
    }

    #[test]
    fn test_parse_stacktrace() {
        let mut reader = JfrReader::new(
            File::open(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/async-profiler-wall.jfr"),
            )
            .unwrap(),
        );
        let (mut reader, chunk) = reader.chunks().next().unwrap().unwrap();
        let event = reader
            .events(&chunk)
            .flatten()
            .filter(|e| e.class.name() == "jdk.ExecutionSample")
            .next()
            .unwrap();
        let (cstr, len) =
            Pool::parse_stacktrace(&event.value().get_field("stackTrace").unwrap()).unwrap();
        let expected = "/usr/lib/aarch64-linux-gnu/libc.so.6
pthread_cond_timedwait
os::PlatformMonitor::wait
Monitor::wait
CompileQueue::get
CompileBroker::compiler_thread_loop
JavaThread::thread_main_inner
Thread::call_run
thread_native_entry
/usr/lib/aarch64-linux-gnu/libc.so.6
/usr/lib/aarch64-linux-gnu/libc.so.6";

        assert_eq!(expected.len(), len);
        assert_eq!(expected, cstr.to_str().unwrap());
    }

    fn query_single<T: FromSql>(sql: &str, column_idx: usize) -> Result<T> {
        let db = Database::new_in_memory()?;
        let conn = db.connect()?;
        conn.register_table_function(&build_table_function_def()?)?;
        let conn = unsafe { Connection::open_from_raw(db.ptr().cast())? };

        let result: duckdb::Result<T> = conn.query_row(sql, [], |row| row.get(column_idx));
        Ok(result?)
    }
}
