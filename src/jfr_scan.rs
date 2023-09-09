use crate::duckdb::bind_info::BindInfo;
use crate::duckdb::bindings::{
    duckdb_bind_info, duckdb_bind_set_error, duckdb_client_context, duckdb_data_chunk,
    duckdb_data_chunk_get_vector, duckdb_free, duckdb_function_info, duckdb_function_set_error,
    duckdb_init_info, duckdb_list_entry, duckdb_list_vector_get_child, duckdb_list_vector_reserve,
    duckdb_list_vector_set_size, duckdb_struct_vector_get_child, duckdb_validity_set_row_invalid,
    duckdb_vector, duckdb_vector_ensure_validity_writable, duckdb_vector_get_validity,
    duckdb_vector_size, idx_t, LogicalTypeId,
};
use crate::duckdb::data_chunk::DataChunk;
use crate::duckdb::file::FileHandle;
use crate::duckdb::function_info::FunctionInfo;
use crate::duckdb::init_info::InitInfo;
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::malloc_struct;
use crate::duckdb::table_function::TableFunction;
use crate::duckdb::vector::Vector;
use crate::jfr_schema::JfrField;
use crate::Result;
use jfrs::reader::event::Accessor;
use jfrs::reader::type_descriptor::{TickUnit, Unit};
use jfrs::reader::value_descriptor::{Primitive, ValueDescriptor};
use jfrs::reader::{Chunk, ChunkReader, JfrReader};
use std::ffi::{c_char, CStr, CString};

use std::mem::{forget, ManuallyDrop};

pub fn build_table_function_def() -> Result<TableFunction> {
    let table_function = TableFunction::new();
    table_function.set_name("jfr_scan")?;
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.set_function(Some(jfr_scan_func));
    table_function.set_init(Some(jfr_scan_init));
    table_function.set_bind(Some(jfr_scan_bind));
    Ok(table_function)
}

unsafe extern "C" fn jfr_scan_bind(context: duckdb_client_context, info: duckdb_bind_info) {
    if let Err(err) = bind(context, info) {
        if let Ok(cstr) = CString::new(err.to_string()) {
            duckdb_bind_set_error(info, cstr.into_raw());
        }
    }
}

unsafe fn bind(context: duckdb_client_context, info: duckdb_bind_info) -> Result<()> {
    let info = BindInfo::from(info);

    let (param0, param1) = (info.get_parameter(0), info.get_parameter(1));
    let filename = param0.get_varchar()?;
    let tablename = param1.get_varchar()?;

    let mut reader = JfrReader::new(FileHandle::open(context, filename));
    // TODO
    let (_, chunk) = reader.chunks().flatten().next().unwrap();

    let (root, _count) = JfrField::from_chunk(&chunk, tablename)?;
    for s in root.children.iter() {
        info.add_result_column(s.name.as_str(), &create_type(s)?)?;
    }

    let bind_data = malloc_struct::<ScanBindData>();
    (*bind_data).filename = filename.as_ptr().cast();
    (*bind_data).tablename = tablename.as_ptr().cast();
    (*bind_data).schema = ManuallyDrop::new(root);
    info.set_bind_data(bind_data.cast(), None);
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
        _ => None,
    }
}

unsafe extern "C" fn jfr_scan_init(info: duckdb_init_info) {
    let info = InitInfo::from(info);
    let init_data = malloc_struct::<ScanInitData>();
    let d = init_data.as_mut().unwrap();
    d.done = false;
    d.chunk_idx = -1;
    d.offset_in_chunk = 0;
    // d.chunks = vec![];
    info.set_init_data(init_data.cast(), Some(duckdb_free));
}

unsafe extern "C" fn jfr_scan_func(
    context: duckdb_client_context,
    info: duckdb_function_info,
    output_raw: duckdb_data_chunk,
) {
    if let Err(err) = scan(context, info, output_raw) {
        duckdb_function_set_error(info, CString::new(err.to_string()).unwrap().into_raw());
    }
}

unsafe fn scan(
    context: duckdb_client_context,
    info: duckdb_function_info,
    output_raw: duckdb_data_chunk,
) -> Result<()> {
    let info = FunctionInfo::from(info);
    let output = DataChunk::from(output_raw);
    let init_data = info.get_init_data::<ScanInitData>().as_mut().unwrap();
    let bind_data = info.get_bind_data::<ScanBindData>().as_ref().unwrap();

    if init_data.done {
        output.set_size(0);
        return Ok(());
    }

    let schema = &bind_data.schema;
    let filename = CStr::from_ptr(bind_data.filename);
    let filename_rs = filename.to_str()?;
    let tablename = CStr::from_ptr(bind_data.tablename);
    let tablename_rs = tablename.to_str()?;

    // Read all data into memory first
    if init_data.chunk_idx < 0 {
        // let mut reader = JfrReader::new(File::open(filename_rs)?);
        let mut reader = JfrReader::new(FileHandle::open(context, filename_rs));
        // let mut reader = JfrReader::new(Cursor::new(SAMPLE_FILE));
        let mut chunks = vec![];
        for chunk in reader.chunks() {
            let (r, c) = chunk?;
            chunks.push((ManuallyDrop::new(r), ManuallyDrop::new(c)));
        }
        if chunks.is_empty() {
            init_data.done = true;
            output.set_size(0);
            return Ok(());
        }
        init_data.chunks = chunks.as_mut_ptr();
        forget(chunks);
        init_data.chunk_idx = 0;
    }

    let vector_size = duckdb_vector_size() as usize;
    let mut children_idx = vec![0usize; 1024]; // TODO grow
    let mut vector_pool = vec![None; 1024]; // TODO grow

    let chunk_idx = init_data.chunk_idx;
    let mut row = 0;

    let (mut reader, chunk) = init_data.chunks.offset(chunk_idx).read();
    for event in reader.events_from_offset(&chunk, init_data.offset_in_chunk) {
        let event = event.unwrap();
        if event.class.name() != tablename_rs {
            continue;
        }
        if row >= vector_size {
            init_data.offset_in_chunk = event.byte_offset;
            output.set_size(row);
            // forget(reader);
            // forget(chunk);
            return Ok(());
        }

        for (i, field) in schema.children.iter().enumerate() {
            if let Some(accessor) = event.value().get_field(field.name.as_str()) {
                populate_column(
                    row,
                    duckdb_data_chunk_get_vector(output_raw, i as idx_t),
                    &output,
                    &chunk,
                    field,
                    accessor,
                    &mut children_idx,
                    &mut vector_pool,
                );
            } else {
                set_null(
                    duckdb_data_chunk_get_vector(output_raw, i as idx_t),
                    row,
                    field,
                    &mut vector_pool,
                    &mut children_idx,
                );
            }
        }
        row += 1;
    }

    // forget(reader);
    // forget(chunk);

    init_data.done = true;
    output.set_size(row);
    Ok(())
}

unsafe fn set_null(
    vector: duckdb_vector,
    row_idx: usize,
    field: &JfrField,
    vector_pool: &mut Vec<Option<duckdb_vector>>,
    children_idx: &mut Vec<usize>,
) {
    duckdb_vector_ensure_validity_writable(vector);
    duckdb_validity_set_row_invalid(duckdb_vector_get_validity(vector), row_idx as u64);

    // Struct children have their own validity bitmaps.
    // Hence we also need to set validity mask for them.
    // Struct children's validity is an AND of parent validity and the child's validity.
    // refs: https://arrow.apache.org/docs/12.0/format/Columnar.html#struct-validity
    if !field.children.is_empty() && !field.is_array {
        for (i, s) in field.children.iter().filter(|s| s.valid).enumerate() {
            // println!("set_null_recursive: idx: {}, name: {}, type: {}", s.idx, s.name, s.type_name);
            if vector_pool[s.idx].is_none() {
                vector_pool[s.idx] = Some(duckdb_struct_vector_get_child(vector, i as idx_t));
            }
            let child_vector = vector_pool[s.idx].unwrap();
            set_null(child_vector, row_idx, s, vector_pool, children_idx);
        }
    }
}

unsafe fn populate_column(
    row_idx: usize,
    vector: duckdb_vector,
    output: &DataChunk,
    chunk: &Chunk,
    child: &JfrField,
    accessor: Accessor<'_>,
    children_idx: &mut Vec<usize>,
    vector_pool: &mut Vec<Option<duckdb_vector>>,
) {
    match accessor.resolve().map(|a| a.value) {
        Some(ValueDescriptor::Primitive(p)) => match p {
            Primitive::Integer(v) => assign(vector, row_idx, *v),
            Primitive::Long(v) => {
                let v = match (child.tick_unit, child.unit) {
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
                Vector::from(vector).assign_string_element_len(row_idx, v.string.as_ptr(), v.len);
            }
            Primitive::Boolean(v) => assign(vector, row_idx, *v),
            Primitive::Short(v) => assign(vector, row_idx, *v),
            Primitive::Byte(v) => assign(vector, row_idx, *v),
            Primitive::NullString => set_null(vector, row_idx, child, vector_pool, children_idx),
            Primitive::String(s) => {
                Vector::from(vector).assign_string_element_len(row_idx, s.string.as_ptr(), s.len);
            }
        },
        Some(ValueDescriptor::Object(obj)) => {
            for (ii, (i, s)) in child
                .children
                .iter()
                .enumerate()
                .map(|(i, s)| (i, s))
                .filter(|(_, s)| s.valid)
                .enumerate()
            {
                if vector_pool[s.idx].is_none() {
                    vector_pool[s.idx] = Some(duckdb_struct_vector_get_child(vector, ii as u64));
                }
                let child_vector = vector_pool[s.idx].unwrap();
                populate_column(
                    row_idx,
                    child_vector,
                    output,
                    chunk,
                    s,
                    Accessor::new(chunk, &obj.fields[i]),
                    children_idx,
                    vector_pool,
                );
            }
        }
        Some(ValueDescriptor::Array(arr)) => {
            let child_vector = duckdb_list_vector_get_child(vector);
            let child_offset = children_idx[child.idx];
            duckdb_list_vector_reserve(vector, (child_offset + arr.len()) as u64);
            for (i, v) in arr.iter().enumerate() {
                populate_column(
                    child_offset + i,
                    child_vector,
                    output,
                    chunk,
                    child,
                    Accessor::new(chunk, v),
                    children_idx,
                    vector_pool,
                );
            }
            Vector::from(vector)
                .get_data::<duckdb_list_entry>().add(row_idx)
                .write(duckdb_list_entry {
                    length: arr.len() as u64,
                    offset: child_offset as u64,
                });
            children_idx[child.idx] = child_offset + arr.len();
            duckdb_list_vector_set_size(vector, (child_offset + arr.len()) as u64);
        }
        _ => {
            set_null(vector, row_idx, child, vector_pool, children_idx);
        }
    }
}

unsafe fn assign<T>(vector: duckdb_vector, row_idx: usize, value: T) {
    let vector = Vector::from(vector);
    vector.get_data::<T>().add(row_idx).write(value);
}

struct ScanInitData {
    done: bool,
    chunks: *mut (ManuallyDrop<ChunkReader>, ManuallyDrop<Chunk>),
    chunk_idx: isize,
    offset_in_chunk: u64,
}

struct ScanBindData {
    filename: *const c_char,
    tablename: *const c_char,
    schema: ManuallyDrop<JfrField>,
}
