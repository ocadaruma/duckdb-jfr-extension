mod duckdb;
mod schema;

use crate::duckdb::bind_info::BindInfo;
use crate::duckdb::bindings::{duckdb_client_context, LogicalTypeId};
use crate::duckdb::data_chunk::DataChunk;
use crate::duckdb::function_info::FunctionInfo;
use crate::duckdb::init_info::InitInfo;
use crate::duckdb::logical_type::LogicalType;
use crate::duckdb::table_function::TableFunction;
use crate::duckdb::vector::Vector;
use crate::duckdb::Database;
use crate::schema::TableStruct;
use jfrs::reader::event::{Accessor, Event};
use jfrs::reader::type_descriptor::{TickUnit, TypeDescriptor, TypePool, Unit};
use jfrs::reader::value_descriptor::{Primitive, ValueDescriptor};
use jfrs::reader::{Chunk, ChunkReader, JfrReader};
use libduckdb_sys::{duckdb_bind_info, duckdb_bind_set_error, duckdb_data_chunk, duckdb_data_chunk_get_vector, duckdb_database, duckdb_free, duckdb_function_info, duckdb_function_set_error, duckdb_init_info, duckdb_library_version, duckdb_list_entry, duckdb_list_vector_get_child, duckdb_list_vector_reserve, duckdb_list_vector_set_size, duckdb_malloc, duckdb_struct_vector_get_child, duckdb_validity_set_row_invalid, duckdb_vector, duckdb_vector_ensure_validity_writable, duckdb_vector_get_validity, duckdb_vector_size, idx_t};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt::format;
use std::fs::File;
use std::io::Cursor;
use std::mem::{forget, size_of, ManuallyDrop};
use std::ptr::{null_mut, slice_from_raw_parts_mut};
use std::slice::from_raw_parts_mut;
use anyhow::anyhow;
use crate::duckdb::file::FileHandle;

type Result<T> = anyhow::Result<T>;

// TODO:
// - multi chunk
// - CString-pool fails with non-constant-pool-String
// - error handlings
// - interval support
// - projection pushdown
// - cleanup comments

const SAMPLE_FILE: &[u8] = include_bytes!(
    "/Users/hokada/develop/src/github.com/moditect/jfr-analytics/src/test/resources/async-profiler-wall.jfr"
);

#[no_mangle]
pub unsafe extern "C" fn libduckdb_jfr_extension_init(db: *mut c_void) {
    init(db).expect("init failed");
}

unsafe fn init(db: *mut c_void) -> Result<()> {
    let db = Database::from(db);
    let conn = db.connect()?;
    let table_function = build_table_function_def();
    conn.register_table_function(&table_function)?;
    jfr_create_view(
        conn.ptr().cast(),
        CString::new("/Users/hokada/develop/src/github.com/moditect/jfr-analytics/src/test/resources/async-profiler-wall.jfr")?.into_raw(),
        CString::new("jdk.ExecutionSample")?.into_raw(),
    );
    println!("init!");
    Ok(())
}

#[no_mangle]
pub extern "C" fn libduckdb_jfr_extension_version() -> *const c_char {
    unsafe { duckdb_library_version() }
}

extern "C" {
    fn jfr_create_view(conn: *mut c_void, filename: *const c_char, tablename: *const c_char);
}

fn build_table_function_def() -> TableFunction {
    let table_function = TableFunction::new();
    table_function.set_name("jfr_scan");
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.add_parameter(&LogicalType::new(LogicalTypeId::Varchar));
    table_function.set_function(Some(jfr_scan_func));
    table_function.set_init(Some(jfr_scan_init));
    table_function.set_bind(Some(jfr_scan_bind));
    table_function
}

unsafe extern "C" fn jfr_scan_bind(info: duckdb_bind_info) {
    if let Err(err) = bind(info) {
        duckdb_bind_set_error(info, CString::new(err.to_string()).unwrap().into_raw());
    }
}

unsafe fn bind(info: duckdb_bind_info) -> Result<()> {
    let info = BindInfo::from(info);
    // info.add_result_column("mycolumn", LogicalType::new(LogicalTypeId::Varchar));
    let (param0, param1) = (info.get_parameter(0), info.get_parameter(1));
    let filename = param0.get_varchar()?;
    let tablename = param1.get_varchar()?;

    // let mut reader = JfrReader::new(File::open(filename).unwrap());
    let mut reader = JfrReader::new(Cursor::new(SAMPLE_FILE));
    let (_, chunk) = reader.chunks().flatten().next().unwrap();

    let strukt = TableStruct::from_chunk(&chunk, tablename);
    for s in strukt.children.iter() {
        info.add_result_column(s.name.as_str(), &create_type(s)?);
    }

    let bind_data = malloc_struct::<JfrBindData>();
    (*bind_data).filename = filename.as_ptr().cast();
    (*bind_data).tablename = tablename.as_ptr().cast();
    (*bind_data).schema = ManuallyDrop::new(strukt);
    info.set_bind_data(bind_data.cast(), None);
    Ok(())
}

fn create_type(strukt: &TableStruct) -> Result<LogicalType> {
    match map_primitive_type(strukt) {
        Some(t) => Ok(t),
        None => {
            let mut shape = vec![];
            for s in strukt.children.iter() {
                // TODO
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

fn map_primitive_type(strukt: &TableStruct) -> Option<LogicalType> {
    match strukt.type_name.as_str() {
        "int" => Some(LogicalType::new(LogicalTypeId::Integer)),
        "long" => match (strukt.tick_unit, strukt.unit) {
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
        _ => None
    }
}

unsafe extern "C" fn jfr_scan_init(info: duckdb_init_info) {
    let info = InitInfo::from(info);
    let init_data = malloc_struct::<JfrInitData>();
    let d = init_data.as_mut().unwrap();
    d.done = false;
    d.chunk_idx = -1;
    d.offset_in_chunk = 0;
    // d.chunks = vec![];
    info.set_init_data(init_data.cast(), Some(duckdb_free));
}

unsafe extern "C" fn jfr_scan_func(context: duckdb_client_context, info: duckdb_function_info, output_raw: duckdb_data_chunk) {
    if let Err(err) = scan(context, info, output_raw) {
        duckdb_function_set_error(info, CString::new(err.to_string()).unwrap().into_raw());
    }
}

unsafe fn scan(context: duckdb_client_context, info: duckdb_function_info, output_raw: duckdb_data_chunk) -> Result<()> {
    let info = FunctionInfo::from(info);
    let output = DataChunk::from(output_raw);
    let init_data = info.get_init_data::<JfrInitData>().as_mut().unwrap();
    let bind_data = info.get_bind_data::<JfrBindData>().as_ref().unwrap();

    if (init_data).done {
        output.set_size(0);
        return Ok(());
    }

    let schema = &bind_data.schema;
    let filename = CStr::from_ptr(bind_data.filename);
    let filename_rs = filename.to_str().unwrap();
    let tablename = CStr::from_ptr(bind_data.tablename);
    let tablename_rs = tablename.to_str().unwrap();

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
        if chunks.len() == 0 {
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
    let field_names = chunk
        .metadata
        .type_pool
        .get_types()
        .filter(|t| t.name() == tablename_rs)
        .flat_map(|t| t.fields.iter().map(|f| f.name()))
        .collect::<Vec<_>>();
    // let field_names = vec!["startTime", "stackTrace"];
    // println!("field_names: {:?}, offset: {}", field_names, init_data.offset_in_chunk);
    // let events: Vec<Event<'_>> = reader.events_from(&chunk, init_data.offset_in_chunk)
    //     .flatten()
    //     .filter(|e| e.class.name() == tablename_rs)
    //     .collect();
    // let mut cstrs = FxHashMap::<u64, CString>::default();
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

        for col in 0..output.get_column_count() {
            // println!("col: {}, name: {}", col, field_names[col as usize].to_string());
            populate_column(
                row,
                duckdb_data_chunk_get_vector(output_raw, col as idx_t),
                &output,
                &chunk,
                &schema.children[col],
                event
                    .value()
                    .get_field(field_names[col])
                    .expect(format!("field {} not found", field_names[col]).as_str()),
                &mut children_idx,
                &mut vector_pool,
            );
        }
        row += 1;
    }

    // forget(reader);
    // forget(chunk);

    init_data.done = true;
    output.set_size(row);
    Ok(())
}

unsafe fn set_null(vector: duckdb_vector, row_idx: usize) {
    duckdb_vector_ensure_validity_writable(vector);
    let idx = duckdb_vector_get_validity(vector);
    duckdb_validity_set_row_invalid(idx, row_idx as u64);
}

unsafe fn set_null_recursive(
    vector: duckdb_vector,
    row_idx: usize,
    strukt: &TableStruct,
    vector_pool: &mut Vec<Option<duckdb_vector>>,
) {
    if strukt.children.len() > 0 {
        for (i, s) in strukt.children.iter().filter(|s| s.valid).enumerate() {
            if vector_pool[s.idx].is_none() {
                vector_pool[s.idx] = Some(duckdb_struct_vector_get_child(vector, i as idx_t));
            }
            let child_vector = vector_pool[s.idx].unwrap();
            set_null_recursive(child_vector, row_idx, s, vector_pool);
        }
    } else {
        set_null(vector, row_idx);
    }
}

unsafe fn populate_column(
    row_idx: usize,
    vector: duckdb_vector,
    output: &DataChunk,
    chunk: &Chunk,
    strukt: &TableStruct,
    accessor: Accessor<'_>,
    children_idx: &mut Vec<usize>,
    vector_pool: &mut Vec<Option<duckdb_vector>>,
) {
    match accessor.resolve().map(|a| a.value) {
        Some(ValueDescriptor::Primitive(p)) => {
            match p {
                Primitive::Integer(v) => assign(vector, row_idx, *v),
                Primitive::Long(v) => {
                    let v = match (strukt.tick_unit, strukt.unit) {
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
                    Vector::from(vector).assign_string_element_len(
                        row_idx,
                        v.string.as_ptr(),
                        v.len,
                    );
                }
                Primitive::Boolean(v) => assign(vector, row_idx, *v),
                Primitive::Short(v) => assign(vector, row_idx, *v),
                Primitive::Byte(v) => assign(vector, row_idx, *v),
                Primitive::NullString => set_null_recursive(vector, row_idx, strukt, vector_pool),
                Primitive::String(s) => {
                    Vector::from(vector).assign_string_element_len(
                        row_idx,
                        s.string.as_ptr(),
                        s.len,
                    );
                }
            }
        }
        Some(ValueDescriptor::Object(obj)) => {
            for (ii, (i, s)) in strukt.children.iter().enumerate().map(|(i, s)| (i, s)).filter(|(_, s)| s.valid).enumerate() {
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
            let child_offset = children_idx[strukt.idx];
            duckdb_list_vector_reserve(vector, (child_offset + arr.len()) as u64);
            for (i, v) in arr.iter().enumerate() {
                populate_column(
                    child_offset + i,
                    child_vector,
                    output,
                    chunk,
                    strukt,
                    Accessor::new(chunk, v),
                    children_idx,
                    vector_pool,
                );
            }
            Vector::from(vector)
                .get_data::<duckdb_list_entry>()
                .offset(row_idx as isize)
                .write(duckdb_list_entry {
                    length: arr.len() as u64,
                    offset: child_offset as u64,
                });
            children_idx[strukt.idx] = child_offset + arr.len();
            duckdb_list_vector_set_size(vector, (child_offset + arr.len()) as u64);
        }
        _ => {
            set_null_recursive(vector, row_idx, strukt, vector_pool);
        }
    }
}

unsafe fn assign<T>(vector: duckdb_vector, row_idx: usize, value: T) {
    let mut vector = Vector::from(vector);
    vector.get_data::<T>().offset(row_idx as isize).write(value);
}

struct JfrInitData {
    done: bool,
    chunks: *mut (ManuallyDrop<ChunkReader>, ManuallyDrop<Chunk>),
    chunk_idx: isize,
    offset_in_chunk: u64,
}

struct JfrBindData {
    filename: *const c_char,
    tablename: *const c_char,
    schema: ManuallyDrop<TableStruct>,
}

pub unsafe fn malloc_struct<T>() -> *mut T {
    duckdb_malloc(size_of::<T>()).cast::<T>()
}
