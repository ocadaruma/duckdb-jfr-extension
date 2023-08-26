mod schema;

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt::format;
use std::fs::File;
use std::mem::{forget, ManuallyDrop};
use std::ptr::{null_mut, slice_from_raw_parts_mut};
use std::slice::from_raw_parts_mut;
use duckdb_extension_framework::{check, Connection, Database, DataChunk, LogicalType, LogicalTypeId, malloc_struct, Vector};
use duckdb_extension_framework::duckly::{duckdb_bind_info, duckdb_column_logical_type, duckdb_connect, duckdb_connection, duckdb_data_chunk, duckdb_data_chunk_get_vector, duckdb_destroy_logical_type, duckdb_free, duckdb_function_info, duckdb_get_type_id, duckdb_init_info, duckdb_library_version, duckdb_list_entry, duckdb_list_vector_get_child, duckdb_list_vector_reserve, duckdb_list_vector_set_size, duckdb_logical_type, duckdb_struct_type_child_count, duckdb_struct_type_child_name, duckdb_struct_vector_get_child, DUCKDB_TYPE_DUCKDB_TYPE_STRUCT, duckdb_validity_set_row_invalid, duckdb_vector, duckdb_vector_ensure_validity_writable, duckdb_vector_get_column_type, duckdb_vector_get_data, duckdb_vector_get_validity, duckdb_vector_size};
use duckdb_extension_framework::table_functions::{BindInfo, FunctionInfo, InitInfo, TableFunction};
use jfrs::reader::event::{Accessor, Event};
use jfrs::reader::{Chunk, ChunkReader, JfrReader};
use jfrs::reader::type_descriptor::{TickUnit, TypeDescriptor, TypePool, Unit};
use jfrs::reader::value_descriptor::{Primitive, ValueDescriptor};
use rustc_hash::FxHashMap;
use crate::schema::TableStruct;

// TODO:
// - multi chunk
// - CString-pool fails with non-constant-pool-String
// - error handlings
// - interval support

#[no_mangle]
pub unsafe extern "C" fn jfr_init(db: *mut c_void) {
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
        CString::new("/Users/hokada/Downloads/LNMSGKF1574.nhnjp.ism-profile-65367-20230806-175232.wall-clock.jfr").unwrap().into_raw(),
        CString::new("jdk.ExecutionSample").unwrap().into_raw(),
    );
    println!("init!");
    Ok(())
}

#[no_mangle]
pub extern "C" fn jfr_version() -> *const c_char {
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
    let info = BindInfo::from(info);
    // info.add_result_column("mycolumn", LogicalType::new(LogicalTypeId::Varchar));
    let filename = info.get_parameter(0).get_varchar();
    let filename_rs = filename.to_str().unwrap();
    let tablename = info.get_parameter(1).get_varchar();
    let tablename_rs = tablename.to_str().unwrap();

    let mut reader = JfrReader::new(File::open(filename_rs).unwrap());
    let (_, chunk) = reader.chunks().flatten().next().unwrap();

    let strukt = TableStruct::from_chunk(&chunk, tablename_rs);
    for s in strukt.children.iter() {
        info.add_result_column(
            s.name.as_str(),
            create_type(s));
    }

    let bind_data = malloc_struct::<JfrBindData>();
    (*bind_data).filename = filename.into_raw();
    (*bind_data).tablename = tablename.into_raw();
    (*bind_data).schema = ManuallyDrop::new(strukt);
    info.set_bind_data(bind_data.cast(), None);
}

fn create_type(strukt: &TableStruct) -> LogicalType {
    match map_primitive_type(strukt) {
        Some(t) => t,
        None => {
            let mut shape = vec![];
            for s in strukt.children.iter() {
                let t = create_type(s);
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
        "long" => {
            match (strukt.tick_unit, strukt.unit) {
                (Some(TickUnit::Timestamp), _) |
                (_, Some(Unit::EpochNano | Unit::EpochMilli | Unit::EpochSecond)) =>
                    Some(LogicalType::new(LogicalTypeId::Timestamp)),
                _ => Some(LogicalType::new(LogicalTypeId::Bigint)),
            }
        }
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

unsafe extern "C" fn jfr_scan_func(
    info: duckdb_function_info,
    output_raw: duckdb_data_chunk
) {
    let info = FunctionInfo::from(info);
    let output = DataChunk::from(output_raw);
    let init_data = info.get_init_data::<JfrInitData>().as_mut().unwrap();
    let bind_data = info.get_bind_data::<JfrBindData>().as_ref().unwrap();

    if (init_data).done {
        output.set_size(0);
        return;
    }

    let schema = &bind_data.schema;
    let filename = CStr::from_ptr(bind_data.filename);
    let filename_rs = filename.to_str().unwrap();
    let tablename = CStr::from_ptr(bind_data.tablename);
    let tablename_rs = tablename.to_str().unwrap();

    // Read all data into memory first
    if init_data.chunk_idx < 0 {
        let mut reader = JfrReader::new(File::open(filename_rs).unwrap());
        let mut chunks: Vec<(ManuallyDrop<ChunkReader>, ManuallyDrop<Chunk>)> = reader
            .chunks()
            .flatten()
            .map(|(r, c)| (ManuallyDrop::new(r), ManuallyDrop::new(c)))
            .collect();
        if chunks.len() == 0 {
            init_data.done = true;
            output.set_size(0);
            return;
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
    let field_names = chunk.metadata.type_pool
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
    let mut cstrs = FxHashMap::<u64, CString>::default();
    for event in reader.events_from(&chunk, init_data.offset_in_chunk) {
        let event = event.unwrap();
        if event.class.name() != tablename_rs {
            continue;
        }
        if row >= vector_size {
            init_data.offset_in_chunk = event.byte_offset;
            output.set_size(row as u64);
            // forget(reader);
            // forget(chunk);
            return;
        }

        for col in 0..output.get_column_count() {
            // println!("col: {}, name: {}", col, field_names[col as usize].to_string());
            populate_column(
                row,
                duckdb_data_chunk_get_vector(output_raw, col),
                &output,
                &chunk,
                &schema.children[col as usize],
                event.value().get_field(field_names[col as usize])
                    .expect(format!("field {} not found", field_names[col as usize]).as_str()),
                &mut children_idx,
                &mut cstrs,
                &mut vector_pool);
        }
        row += 1;
    }

    // forget(reader);
    // forget(chunk);

    init_data.done = true;
    output.set_size(row as u64);
}

struct Sub {
    foo: Vec<i32>,
    bar: f64,
}

unsafe fn set_null(vector: duckdb_vector, row_idx: usize) {
    duckdb_vector_ensure_validity_writable(vector);
    let idx = duckdb_vector_get_validity(vector);
    duckdb_validity_set_row_invalid(idx, row_idx as u64);
}

unsafe fn set_null_recursive(vector: duckdb_vector,
                             row_idx: usize,
                             strukt: &TableStruct,
                             vector_pool: &mut Vec<Option<duckdb_vector>>) {
    if strukt.children.len() > 0 {
        for (i, s) in strukt.children.iter().enumerate() {
            if vector_pool[s.idx].is_none() {
                vector_pool[s.idx] = Some(duckdb_struct_vector_get_child(vector, i as u64));
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
    cstrs: &mut FxHashMap<u64, CString>,
    vector_pool: &mut Vec<Option<duckdb_vector>>) {
    match accessor.get_resolved().map(|v| v.value) {
        Some(ValueDescriptor::Primitive(p)) => {
            // println!("primitive!!!: {:?}", p);
            match p {
                Primitive::Integer(v) => assign(vector, row_idx, *v),
                Primitive::Long(v) => {
                    let v = match (strukt.tick_unit, strukt.unit) {
                        (Some(TickUnit::Timestamp), _) => {
                            let ticks_per_nanos = (chunk.header.ticks_per_second as f64) / 1_000_000_000.0;
                            (chunk.header.start_time_nanos + ((*v - chunk.header.start_ticks) as f64 / ticks_per_nanos) as i64) / 1_000
                        }
                        (_, Some(Unit::EpochNano)) => *v / 1_000,
                        (_, Some(Unit::EpochMilli)) => *v * 1_000,
                        (_, Some(Unit::EpochSecond)) => *v * 1_000_000,
                        _ => *v,
                    };
                    assign(vector, row_idx, v)
                },
                Primitive::Float(v) => assign(vector, row_idx, *v),
                Primitive::Double(v) => assign(vector, row_idx, *v),
                Primitive::Character(v) => {
                    let cs = CString::new(v.to_string()).unwrap();
                    Vector::<()>::from(vector).assign_string_element_len(
                        row_idx as u64, cs.as_ptr(), v.to_string().len() as u64);
                }
                Primitive::Boolean(v) => assign(vector, row_idx, *v),
                Primitive::Short(v) => assign(vector, row_idx, *v),
                Primitive::Byte(v) => assign(vector, row_idx, *v),
                Primitive::NullString => set_null_recursive(vector, row_idx, strukt, vector_pool),
                Primitive::String(s) => {
                    let ptr = s.as_ptr() as u64;
                    if let Some(cs) = cstrs.get(&ptr) {
                        Vector::<()>::from(vector).assign_string_element_len(
                            row_idx as u64, cs.as_ptr(), s.len() as u64);
                    } else {
                        let cs = CString::new(s.as_str()).unwrap();
                        Vector::<()>::from(vector).assign_string_element_len(
                            row_idx as u64, cs.as_ptr(), s.len() as u64);
                        cstrs.insert(ptr, cs);
                    }
                }
            }
        }
        Some(ValueDescriptor::Object(obj)) => {
            // println!("obj!!!");
            for (i, s) in strukt.children.iter().enumerate() {
                // println!("field_name: {}, type_name: {}", name_rs, type_desc.name());
                // println!("type: {}, field: {}, field_idx: {}, jfr_field_idx: {}", type_desc.name(), name_rs, field_idx, jfr_field_idx);
                if vector_pool[s.idx].is_none() {
                    vector_pool[s.idx] = Some(duckdb_struct_vector_get_child(vector, i as u64));
                }
                let child_vector = vector_pool[s.idx].unwrap();
                if let Some(acc) = Accessor::new(chunk, &obj.fields[i]).get_resolved() {
                    populate_column(
                        row_idx,
                        child_vector,
                        output,
                        chunk,
                        s,
                        acc,
                        children_idx,
                        cstrs,
                        vector_pool);
                } else {
                    set_null_recursive(child_vector, row_idx, s, vector_pool);
                }
            }
        }
        Some(ValueDescriptor::Array(arr)) => {
            // println!("array!!!");
            let child_vector = duckdb_list_vector_get_child(vector);
            let child_offset = children_idx[strukt.idx];
            duckdb_list_vector_reserve(vector, (child_offset + arr.len()) as u64);
            for (i, v) in arr.iter().enumerate() {
                // println!("{} : {}", field_selector, child_offset + i);
                if let Some(acc) = Accessor::new(chunk, v).get_resolved() {
                    populate_column(
                        child_offset + i,
                        child_vector,
                        output,
                        chunk,
                        strukt,
                        acc,
                        children_idx,
                        cstrs,
                        vector_pool);
                } else {
                    set_null_recursive(child_vector, child_offset + i, strukt, vector_pool);
                }
            }
            Vector::<duckdb_list_entry>::from(vector).get_data().offset(row_idx as isize).write(duckdb_list_entry {
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
    let mut vector = Vector::<T>::from(vector);
    vector.get_data().offset(row_idx as isize).write(value);
}

struct JfrInitData {
    done: bool,
    chunks: *mut (ManuallyDrop<ChunkReader>, ManuallyDrop<Chunk>),
    chunk_idx: isize,
    offset_in_chunk: u64,
}

struct JfrBindData {
    filename: *mut c_char,
    tablename: *mut c_char,
    schema: ManuallyDrop<TableStruct>,
}
