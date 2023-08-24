use jfrs::reader::Chunk;
use jfrs::reader::type_descriptor::{TypeDescriptor, TypePool};

#[derive(Debug)]
pub struct TableStruct {
    pub idx: usize,
    pub is_array: bool,
    pub type_name: String,
    pub children: Vec<TableStruct>,
}

impl TableStruct {
    pub fn new(type_name: String) -> Self {
        Self {
            idx: 0,
            is_array: false,
            type_name,
            children: vec![],
        }
    }

    pub fn from_chunk(chunk: &Chunk, type_name: &str) -> Self {
        let mut root = Self::new(type_name.to_string());
        let tpe = chunk.metadata
            .type_pool
            .get_types()
            .filter(|t| t.name() == type_name)
            .next()
            .unwrap();
        Self::traverse(root.idx, &mut root, tpe, &chunk.metadata.type_pool, vec![]);
        root
    }

    fn traverse(
        idx: usize,
        strukt: &mut TableStruct,
        tpe: &TypeDescriptor,
        type_pool: &TypePool,
        visited_classes: Vec<i64>) -> usize {
        // recursion guard
        if visited_classes.contains(&tpe.class_id) {
            return 1;
        }
        // primitive
        if tpe.fields.is_empty() {
            return 1;
        }
        let mut v = visited_classes.clone();
        v.push(tpe.class_id);

        let mut idx = idx;
        let mut child_count = 1;
        for field in tpe.fields.iter() {
            let next_tpe = type_pool.get(field.class_id).unwrap();
            let mut child = TableStruct::new(next_tpe.name().to_string());
            child.idx = idx + child_count;
            child.is_array = field.array_type;
            child_count += Self::traverse(child.idx, &mut child, next_tpe, type_pool, v.clone());
            strukt.children.push(child);
        }
        child_count
    }
}

// mod tests {
//     use std::fs::File;
//     use std::process::id;
//     use jfrs::reader::JfrReader;
//     use jfrs::reader::type_descriptor::{FieldDescriptor, TypeDescriptor, TypePool};
//     use crate::schema::TableStruct;
//
//     #[test]
//     fn test_schema() {
//         let path = "/Users/hokada/develop/src/github.com/moditect/jfr-analytics/src/test/resources/async-profiler-wall.jfr";
//         // get the first element of the iterator
//         let (_, chunk) = JfrReader::new(File::open(path).unwrap())
//             .chunks()
//             .flatten()
//             .next()
//             .unwrap();
//
//         let tpe = chunk.metadata
//             .type_pool
//             .get_types()
//             .filter(|t| t.name() == "jdk.ExecutionSample")
//             .next()
//             .unwrap();
//
//         let mut root = TableStruct::new();
//         traverse(0, &mut root, tpe, &chunk.metadata.type_pool, vec![]);
//         println!("{:#?}", root);
//     }
//
//     fn traverse(
//         idx: usize,
//         strukt: &mut TableStruct,
//         tpe: &TypeDescriptor,
//         type_pool: &TypePool,
//         visited_classes: Vec<i64>) -> usize {
//         // recursion guard
//         if visited_classes.contains(&tpe.class_id) {
//             return 1;
//         }
//         // primitive
//         if tpe.fields.is_empty() {
//             return 1;
//         }
//         let mut v = visited_classes.clone();
//         v.push(tpe.class_id);
//
//         let mut idx = idx;
//         let mut child_count = 1;
//         for field in tpe.fields.iter() {
//             let next_tpe = type_pool.get(field.class_id).unwrap();
//             let mut child = TableStruct::new();
//             child.idx = idx + child_count;
//             child.is_array = field.array_type;
//             child_count += traverse(child.idx, &mut child, next_tpe, type_pool, v.clone());
//             strukt.children.push(child);
//         }
//         child_count
//     }
// }
