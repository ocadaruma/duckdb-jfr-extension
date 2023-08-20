use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt::format;
use std::fs::File;
use std::mem::forget;
use std::ptr::{null_mut, slice_from_raw_parts_mut};
use std::slice::from_raw_parts_mut;
use duckdb_extension_framework::{check, Connection, Database, DataChunk, LogicalType, LogicalTypeId, malloc_struct, Vector};
use duckdb_extension_framework::duckly::{duckdb_bind_info, duckdb_column_logical_type, duckdb_connect, duckdb_connection, duckdb_data_chunk, duckdb_data_chunk_get_vector, duckdb_destroy_logical_type, duckdb_free, duckdb_function_info, duckdb_get_type_id, duckdb_init_info, duckdb_library_version, duckdb_list_entry, duckdb_list_vector_get_child, duckdb_list_vector_reserve, duckdb_list_vector_set_size, duckdb_logical_type, duckdb_struct_type_child_count, duckdb_struct_type_child_name, duckdb_struct_vector_get_child, DUCKDB_TYPE_DUCKDB_TYPE_STRUCT, duckdb_validity_set_row_invalid, duckdb_vector, duckdb_vector_ensure_validity_writable, duckdb_vector_get_column_type, duckdb_vector_get_data, duckdb_vector_get_validity, duckdb_vector_size};
use duckdb_extension_framework::table_functions::{BindInfo, FunctionInfo, InitInfo, TableFunction};
use jfrs::reader::event::{Accessor, Event};
use jfrs::reader::{Chunk, ChunkReader, JfrReader};
use jfrs::reader::type_descriptor::{TypeDescriptor, TypePool};
use jfrs::reader::value_descriptor::{Primitive, ValueDescriptor};
use rustc_hash::FxHashMap;

// TODO:
// - multi chunk
// - CString-pool fails with non-constant-pool-String
// - proper child-index management (without using format!)

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
        CString::new("/Users/hokada/develop/src/github.com/moditect/jfr-analytics/src/test/resources/async-profiler-wall.jfr").unwrap().into_raw(),
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
    for chunk in reader.chunks() {
        let (_, chunk) = chunk.unwrap();
        for type_desc in chunk.metadata.type_pool.get_types() {
            if type_desc.name() == tablename_rs {
                for field in type_desc.fields.iter() {
                    let tpe = chunk.metadata.type_pool.get(field.class_id).unwrap();
                    info.add_result_column(
                        field.name(),
                        create_type(&chunk.metadata.type_pool, tpe));
                }
            }
        }
    }

    let bind_data = malloc_struct::<JfrBindData>();
    (*bind_data).filename = filename.into_raw();
    (*bind_data).tablename = tablename.into_raw();
    info.set_bind_data(bind_data.cast(), None);
}

// unsafe extern "C" fn jfr_scan_bind(info: duckdb_bind_info) {
//     let info = BindInfo::from(info);
//     // info.add_result_column("mycolumn", LogicalType::new(LogicalTypeId::Varchar));
//     let filename = info.get_parameter(0).get_varchar();
//     let filename_rs = filename.to_str().unwrap();
//     let tablename = info.get_parameter(1).get_varchar();
//     let tablename_rs = tablename.to_str().unwrap();
//
//     info.add_result_column("startTime", LogicalType::new(LogicalTypeId::Bigint));
//     // info.add_result_column("sampledThread", LogicalType::new_struct_type(&[
//     //     ("osName", LogicalType::new(LogicalTypeId::Varchar)),
//     //     ("osThreadId", LogicalType::new(LogicalTypeId::Bigint)),
//     //     ("javaName", LogicalType::new(LogicalTypeId::Varchar)),
//     //     ("javaThreadId", LogicalType::new(LogicalTypeId::Bigint)),
//     // ]));
//     info.add_result_column(
//         "numbers",
//         LogicalType::new_list_type(&LogicalType::new(LogicalTypeId::Integer)));
//     // info.add_result_column("state", LogicalType::new_struct_type(&[
//     //     ("name", LogicalType::new(LogicalTypeId::Varchar)),
//     // ]));
//
//     let bind_data = malloc_struct::<JfrBindData>();
//     (*bind_data).filename = filename.into_raw();
//     (*bind_data).tablename = tablename.into_raw();
//     info.set_bind_data(bind_data.cast(), None);
// }

fn create_type(type_pool: &TypePool, type_desc: &TypeDescriptor) -> LogicalType {
    match map_primitive_type(type_desc.name()) {
        Some(t) => t,
        None => {
            let mut shape = vec![];
            for field in type_desc.fields.iter() {
                let t = type_pool.get(field.class_id).unwrap();
                // println!("field: {}, type: {}", field.name(), t.name());
                let t = create_type(type_pool, t);
                let t = if field.array_type {
                    LogicalType::new_list_type(&t)
                } else {
                    t
                };
                shape.push((field.name(), t));
            }
            LogicalType::new_struct_type(shape.as_slice())
        }
    }
}

fn map_primitive_type(type_name: &str) -> Option<LogicalType> {
    match type_name {
        "int" => Some(LogicalType::new(LogicalTypeId::Integer)),
        "long" => Some(LogicalType::new(LogicalTypeId::Bigint)),
        "float" => Some(LogicalType::new(LogicalTypeId::Float)),
        "double" => Some(LogicalType::new(LogicalTypeId::Double)),
        "char" => Some(LogicalType::new(LogicalTypeId::Varchar)),
        "boolean" => Some(LogicalType::new(LogicalTypeId::Boolean)),
        "short" => Some(LogicalType::new(LogicalTypeId::Smallint)),
        "byte" => Some(LogicalType::new(LogicalTypeId::Tinyint)),
        "java.lang.String" => Some(LogicalType::new(LogicalTypeId::Varchar)),
        "jdk.types.ClassLoader" => {
            let symbol = LogicalType::new_struct_type(&[("string", LogicalType::new(LogicalTypeId::Varchar))]);
            Some(LogicalType::new_struct_type(&[("name", symbol)]))
        }
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

    let filename = CStr::from_ptr(bind_data.filename);
    let filename_rs = filename.to_str().unwrap();
    let tablename = CStr::from_ptr(bind_data.tablename);
    let tablename_rs = tablename.to_str().unwrap();

    // Read all data into memory first
    if init_data.chunk_idx < 0 {
        let mut reader = JfrReader::new(File::open(filename_rs).unwrap());
        let mut chunks: Vec<(ChunkReader, Chunk)> = reader.chunks().flatten().collect();
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
    let mut children_idx = vec![0usize; 1024];

    let chunk_idx = init_data.chunk_idx;
    let mut row = 0;
    let mut child_offsets = HashMap::<String, usize>::new();

    let (mut reader, chunk) = init_data.chunks.offset(chunk_idx).read();
    let field_names = chunk.metadata.type_pool
        .get_types()
        .filter(|t| t.name() == tablename_rs)
        .flat_map(|t| t.fields.iter().map(|f| f.name()))
        .collect::<Vec<_>>();
    // let field_names = vec!["startTime", "stackTrace"];
    println!("field_names: {:?}, offset: {}", field_names, init_data.offset_in_chunk);
    // let events: Vec<Event<'_>> = reader.events_from(&chunk, init_data.offset_in_chunk)
    //     .flatten()
    //     .filter(|e| e.class.name() == tablename_rs)
    //     .collect();
    let mut cstrs = FxHashMap::<u64, CString>::default();
    let mut logical_type_pool = FxHashMap::<u64, duckdb_logical_type>::default();
    for event in reader.events_from(&chunk, init_data.offset_in_chunk) {
        let event = event.unwrap();
        if event.class.name() != tablename_rs {
            continue;
        }
        if row >= vector_size {
            init_data.offset_in_chunk = event.byte_offset;
            output.set_size(row as u64);
            forget(reader);
            forget(chunk);
            return;
        }
        let mut n = 0;
        for col in 0..output.get_column_count() {
            // println!("col: {}, name: {}", col, field_names[col as usize].to_string());
            populate_column(
                row,
                duckdb_data_chunk_get_vector(output_raw, col),
                &output,
                &chunk.metadata.type_pool,
                &chunk,
                event.value().get_field(field_names[col as usize])
                    .expect(format!("field {} not found", field_names[col as usize]).as_str()),
                &mut children_idx,
                &mut cstrs,
                &mut logical_type_pool);
        }
        row += 1;
    }
    for value in logical_type_pool.values_mut() {
        duckdb_destroy_logical_type(value);
    }

    forget(reader);
    forget(chunk);

    init_data.done = true;
    output.set_size(row as u64);
}

struct Sub {
    foo: Vec<i32>,
    bar: f64,
}

// unsafe extern "C" fn jfr_scan_func(
//     info: duckdb_function_info,
//     output_raw: duckdb_data_chunk
// ) {
//     let info = FunctionInfo::from(info);
//     let output = DataChunk::from(output_raw);
//     let init_data = info.get_init_data::<JfrInitData>();
//     let bind_data = info.get_bind_data::<JfrBindData>();
//
//     if (*init_data).done {
//         output.set_size(0);
//         return;
//     }
//
//     let filename = CStr::from_ptr((*bind_data).filename);
//     let filename_rs = filename.to_str().unwrap();
//     let tablename = CStr::from_ptr((*bind_data).tablename);
//     let tablename_rs = tablename.to_str().unwrap();
//
//     let vector = duckdb_data_chunk_get_vector(output_raw, 1);
//     // duckdb_list_vector_reserve(vector, 1024 * 1024);
//
//     let row_count = 1024;
//     let sub_vector: Vec<i32> = (0..10).collect();
//     let mut child_offset = 0;
//     for row in 0..row_count {
//         output.get_vector::<u64>(0).get_data_as_slice()[row] = 1691936000000 + row as u64;
//         // let vector = duckdb_data_chunk_get_vector(output_raw, 1);
//         let capacity = child_offset + sub_vector.len();
//         duckdb_list_vector_reserve(vector, capacity as u64);
//         let array = duckdb_vector_get_data(duckdb_list_vector_get_child(vector));
//         let slice = from_raw_parts_mut::<i32>(array.cast(), capacity);
//         for (i, n) in sub_vector.iter().enumerate() {
//             slice[child_offset + i] = *n;
//         }
//         Vector::<duckdb_list_entry>::from(vector).get_data_as_slice()[row] = duckdb_list_entry {
//             length: sub_vector.len() as u64,
//             offset: child_offset as u64,
//         };
//         child_offset += sub_vector.len();
//         duckdb_list_vector_set_size(vector, child_offset as u64);
//         // // truncated
//         // let v = duckdb_struct_vector_get_child(vector, 0);
//         // Vector::<bool>::from(v).get_data_as_slice()[row] = true;
//         //
//         // // frames
//         // let v = duckdb_struct_vector_get_child(vector, 1);
//         //
//         // let sub = &sub_vector[row];
//         //
//         // duckdb_list_vector_reserve(v, 5000);
//         // let list_v = duckdb_list_vector_get_child(v);
//         //
//         // for i in 0..sub.len() {
//         //     let subb = &sub[i];
//         //
//         //     // foo >>
//         //     let foo_v = duckdb_struct_vector_get_child(list_v, 0);
//         //     duckdb_list_vector_reserve(foo_v, 5000);
//         //     let foo_list_v = duckdb_list_vector_get_child(foo_v);
//         //     for ii in 0..subb.foo.len() {
//         //         Vector::<i32>::from(foo_list_v).get_data_as_slice()[child_offset_2 + ii] = subb.foo[ii];
//         //     }
//         //     Vector::<duckdb_list_entry>::from(foo_v).get_data_as_slice()[child_offset + i] = duckdb_list_entry {
//         //         length: subb.foo.len() as u64,
//         //         offset: child_offset_2 as u64,
//         //     };
//         //     child_offset_2 += subb.foo.len();
//         //     duckdb_list_vector_set_size(foo_v, (child_offset_2 + subb.foo.len()) as u64);
//         //     // << foo
//         //
//         //     let bar_v = duckdb_struct_vector_get_child(list_v, 1);
//         //     Vector::<f64>::from(bar_v).get_data_as_slice()[child_offset + i] = subb.bar;
//         // }
//         // Vector::<duckdb_list_entry>::from(v).get_data_as_slice()[row] = duckdb_list_entry {
//         //     length: sub.len() as u64,
//         //     offset: child_offset as u64,
//         // };
//         // child_offset += sub.len();
//         // duckdb_list_vector_set_size(v, (child_offset + sub.len()) as u64);
//     }
//     (*init_data).done = true;
//     output.set_size(row_count as u64);
// }

unsafe fn set_null(vector: duckdb_vector, row_idx: usize) {
    duckdb_vector_ensure_validity_writable(vector);
    let idx = duckdb_vector_get_validity(vector);
    duckdb_validity_set_row_invalid(idx, row_idx as u64);
}

unsafe fn set_null_recursive(vector: duckdb_vector,
                             row_idx: usize,
                             logical_type_pool: &mut FxHashMap<u64, duckdb_logical_type>) {
    let mut logical_type = logical_type_pool.entry(vector as u64)
        .or_insert_with(|| duckdb_vector_get_column_type(vector));

    if duckdb_get_type_id(*logical_type) == DUCKDB_TYPE_DUCKDB_TYPE_STRUCT {
        let child_count = duckdb_struct_type_child_count(*logical_type);
        for i in 0..child_count {
            let child = duckdb_struct_vector_get_child(vector, i);
            set_null_recursive(child, row_idx, logical_type_pool);
        }
    } else {
        set_null(vector, row_idx);
    }

    // duckdb_destroy_logical_type(&mut logical_type);
}

unsafe fn populate_column(
    row_idx: usize,
    vector: duckdb_vector,
    output: &DataChunk,
    type_pool: &TypePool,
    chunk: &Chunk,
    accessor: Accessor<'_>,
    children_idx: &mut Vec<usize>,
    cstrs: &mut FxHashMap<u64, CString>,
    logical_type_pool: &mut FxHashMap<u64, duckdb_logical_type>) {
    match accessor.get_resolved().map(|v| v.value) {
        Some(ValueDescriptor::Primitive(p)) => {
            // println!("primitive!!!: {:?}", p);
            match p {
                Primitive::Integer(v) => assign(vector, row_idx, *v),
                Primitive::Long(v) => assign(vector, row_idx, *v),
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
                Primitive::NullString => set_null_recursive(vector, row_idx, logical_type_pool),
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
            let mut logical_type = logical_type_pool.entry(vector as u64)
                .or_insert_with(|| duckdb_vector_get_column_type(vector));
            let cnt = duckdb_struct_type_child_count(*logical_type);
            let mut f = 0;
            for field_idx in 0..cnt {
                // println!("field_name: {}, type_name: {}", name_rs, type_desc.name());
                // println!("type: {}, field: {}, field_idx: {}, jfr_field_idx: {}", type_desc.name(), name_rs, field_idx, jfr_field_idx);
                let child_vector = duckdb_struct_vector_get_child(vector, field_idx);
                if let Some(acc) = Accessor::new(chunk, &obj.fields[field_idx as usize]).get_resolved() {
                    populate_column(
                        row_idx,
                        child_vector,
                        output,
                        type_pool,
                        chunk,
                        acc,
                        children_idx,
                        cstrs,
                        logical_type_pool);
                } else {
                    set_null_recursive(child_vector, row_idx, logical_type_pool);
                }
            }
            // duckdb_destroy_logical_type(&mut logical_type);
        }
        Some(ValueDescriptor::Array(arr)) => {
            // println!("array!!!");
            let child_vector = duckdb_list_vector_get_child(vector);
            let child_offset = children_idx[0];
            duckdb_list_vector_reserve(vector, (child_offset + arr.len()) as u64);
            for (i, v) in arr.iter().enumerate() {
                // println!("{} : {}", field_selector, child_offset + i);
                if let Some(acc) = Accessor::new(chunk, v).get_resolved() {
                    populate_column(
                        child_offset + i,
                        child_vector,
                        output,
                        type_pool,
                        chunk,
                        acc,
                        children_idx,
                        cstrs,
                        logical_type_pool);
                } else {
                    set_null_recursive(child_vector, child_offset + i, logical_type_pool);
                }
            }
            Vector::<duckdb_list_entry>::from(vector).get_data().offset(row_idx as isize).write(duckdb_list_entry {
                length: arr.len() as u64,
                offset: child_offset as u64,
            });
            children_idx[0] = child_offset + arr.len();
            duckdb_list_vector_set_size(vector, (child_offset + arr.len()) as u64);
        }
        _ => {
            set_null_recursive(vector, row_idx, logical_type_pool);
        }
    }
}

unsafe fn assign<T>(vector: duckdb_vector, row_idx: usize, value: T) {
    let mut vector = Vector::<T>::from(vector);
    vector.get_data().offset(row_idx as isize).write(value);
}

struct JfrInitData {
    done: bool,
    chunks: *mut (ChunkReader, Chunk),
    chunk_idx: isize,
    offset_in_chunk: u64,
}

struct JfrBindData {
    filename: *mut c_char,
    tablename: *mut c_char,
}
