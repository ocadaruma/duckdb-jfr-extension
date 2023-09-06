use crate::Result;
use anyhow::anyhow;
use jfrs::reader::type_descriptor::{TickUnit, TypeDescriptor, TypePool, Unit};
use jfrs::reader::Chunk;

#[derive(Debug, Eq, PartialEq)]
pub struct JfrField {
    pub idx: usize,
    pub is_array: bool,
    pub name: String,
    pub type_name: String,
    pub tick_unit: Option<TickUnit>,
    pub unit: Option<Unit>,
    pub children: Vec<JfrField>,
    pub valid: bool,
}

impl JfrField {
    /// Create a JfrField of the type from a Chunk.
    /// Returns the root and the number of fields in the tree.
    pub fn from_chunk(chunk: &Chunk, type_name: &str) -> Result<(Self, usize)> {
        let tpe = chunk
            .metadata
            .type_pool
            .get_types()
            .filter(|t| t.name() == type_name)
            .next()
            .ok_or(anyhow!("type not found: {:?}", type_name))?;
        let mut idx = 0;
        let mut root = Self {
            idx,
            is_array: false,
            name: "root".to_string(),
            type_name: type_name.to_string(),
            tick_unit: None,
            unit: None,
            children: vec![],
            valid: true,
        };
        let mut max_idx = 1;
        Self::traverse(
            &mut idx,
            &mut max_idx,
            &mut root,
            tpe,
            &chunk.metadata.type_pool,
            vec![],
        )?;
        Ok((root, max_idx + 1))
    }

    fn traverse(
        idx: &mut usize,
        max_idx: &mut usize,
        current_field: &mut JfrField,
        tpe: &TypeDescriptor,
        type_pool: &TypePool,
        visited_classes: Vec<i64>,
    ) -> Result<()> {
        *idx += 1;

        // recursion guard
        if visited_classes.contains(&tpe.class_id) {
            current_field.valid = false;
            return Ok(());
        }
        // primitive
        if tpe.fields.is_empty() {
            return Ok(());
        }

        let mut v = visited_classes.clone();
        v.push(tpe.class_id);
        for f in tpe.fields.iter() {
            let next_tpe = type_pool
                .get(f.class_id)
                .ok_or(anyhow!("type not found: {}", f.class_id))?;
            let mut child = Self {
                idx: *idx,
                is_array: f.array_type,
                name: f.name().to_string(),
                type_name: next_tpe.name().to_string(),
                tick_unit: f.tick_unit,
                unit: f.unit,
                children: vec![],
                valid: true,
            };
            *max_idx = *idx.max(max_idx);
            Self::traverse(idx, max_idx, &mut child, next_tpe, type_pool, v.clone())?;
            current_field.children.push(child);
        }
        Ok(())
    }
}

mod tests {
    use crate::jfr_schema::JfrField;
    use jfrs::reader::type_descriptor::TickUnit;
    use jfrs::reader::JfrReader;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_schema() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/async-profiler-wall.jfr");
        let (_, chunk) = JfrReader::new(File::open(path).unwrap())
            .chunks()
            .flatten()
            .next()
            .unwrap();
        let (root, count) = JfrField::from_chunk(&chunk, "jdk.ExecutionSample").unwrap();
        let expected = JfrField {
            idx: 0,
            is_array: false,
            name: "root".to_string(),
            type_name: "jdk.ExecutionSample".to_string(),
            tick_unit: None,
            unit: None,
            children: vec![
                JfrField {
                    idx: 1,
                    is_array: false,
                    name: "startTime".to_string(),
                    type_name: "long".to_string(),
                    tick_unit: Some(TickUnit::Timestamp),
                    unit: None,
                    children: vec![],
                    valid: true,
                },
                JfrField {
                    idx: 2,
                    is_array: false,
                    name: "sampledThread".to_string(),
                    type_name: "java.lang.Thread".to_string(),
                    tick_unit: None,
                    unit: None,
                    children: vec![
                        JfrField {
                            idx: 3,
                            is_array: false,
                            name: "osName".to_string(),
                            type_name: "java.lang.String".to_string(),
                            tick_unit: None,
                            unit: None,
                            children: vec![],
                            valid: true,
                        },
                        JfrField {
                            idx: 4,
                            is_array: false,
                            name: "osThreadId".to_string(),
                            type_name: "long".to_string(),
                            tick_unit: None,
                            unit: None,
                            children: vec![],
                            valid: true,
                        },
                        JfrField {
                            idx: 5,
                            is_array: false,
                            name: "javaName".to_string(),
                            type_name: "java.lang.String".to_string(),
                            tick_unit: None,
                            unit: None,
                            children: vec![],
                            valid: true,
                        },
                        JfrField {
                            idx: 6,
                            is_array: false,
                            name: "javaThreadId".to_string(),
                            type_name: "long".to_string(),
                            tick_unit: None,
                            unit: None,
                            children: vec![],
                            valid: true,
                        },
                    ],
                    valid: true,
                },
                JfrField {
                    idx: 7,
                    is_array: false,
                    name: "stackTrace".to_string(),
                    type_name: "jdk.types.StackTrace".to_string(),
                    tick_unit: None,
                    unit: None,
                    children: vec![
                        JfrField {
                            idx: 8,
                            is_array: false,
                            name: "truncated".to_string(),
                            type_name: "boolean".to_string(),
                            tick_unit: None,
                            unit: None,
                            children: vec![],
                            valid: true,
                        },
                        JfrField {
                            idx: 9,
                            is_array: true,
                            name: "frames".to_string(),
                            type_name: "jdk.types.StackFrame".to_string(),
                            tick_unit: None,
                            unit: None,
                            children: vec![
                                JfrField {
                                    idx: 10,
                                    is_array: false,
                                    name: "method".to_string(),
                                    type_name: "jdk.types.Method".to_string(),
                                    tick_unit: None,
                                    unit: None,
                                    children: vec![
                                        JfrField {
                                            idx: 11,
                                            is_array: false,
                                            name: "type".to_string(),
                                            type_name: "java.lang.Class".to_string(),
                                            tick_unit: None,
                                            unit: None,
                                            children: vec![
                                                JfrField {
                                                    idx: 12,
                                                    is_array: false,
                                                    name: "classLoader".to_string(),
                                                    type_name: "jdk.types.ClassLoader".to_string(),
                                                    tick_unit: None,
                                                    unit: None,
                                                    children: vec![
                                                        JfrField {
                                                            idx: 13,
                                                            is_array: false,
                                                            name: "type".to_string(),
                                                            type_name: "java.lang.Class"
                                                                .to_string(),
                                                            tick_unit: None,
                                                            unit: None,
                                                            children: vec![],
                                                            valid: false,
                                                        },
                                                        JfrField {
                                                            idx: 14,
                                                            is_array: false,
                                                            name: "name".to_string(),
                                                            type_name: "jdk.types.Symbol"
                                                                .to_string(),
                                                            tick_unit: None,
                                                            unit: None,
                                                            children: vec![JfrField {
                                                                idx: 15,
                                                                is_array: false,
                                                                name: "string".to_string(),
                                                                type_name: "java.lang.String"
                                                                    .to_string(),
                                                                tick_unit: None,
                                                                unit: None,
                                                                children: vec![],
                                                                valid: true,
                                                            }],
                                                            valid: true,
                                                        },
                                                    ],
                                                    valid: true,
                                                },
                                                JfrField {
                                                    idx: 16,
                                                    is_array: false,
                                                    name: "name".to_string(),
                                                    type_name: "jdk.types.Symbol".to_string(),
                                                    tick_unit: None,
                                                    unit: None,
                                                    children: vec![JfrField {
                                                        idx: 17,
                                                        is_array: false,
                                                        name: "string".to_string(),
                                                        type_name: "java.lang.String".to_string(),
                                                        tick_unit: None,
                                                        unit: None,
                                                        children: vec![],
                                                        valid: true,
                                                    }],
                                                    valid: true,
                                                },
                                                JfrField {
                                                    idx: 18,
                                                    is_array: false,
                                                    name: "package".to_string(),
                                                    type_name: "jdk.types.Package".to_string(),
                                                    tick_unit: None,
                                                    unit: None,
                                                    children: vec![JfrField {
                                                        idx: 19,
                                                        is_array: false,
                                                        name: "name".to_string(),
                                                        type_name: "jdk.types.Symbol".to_string(),
                                                        tick_unit: None,
                                                        unit: None,
                                                        children: vec![JfrField {
                                                            idx: 20,
                                                            is_array: false,
                                                            name: "string".to_string(),
                                                            type_name: "java.lang.String"
                                                                .to_string(),
                                                            tick_unit: None,
                                                            unit: None,
                                                            children: vec![],
                                                            valid: true,
                                                        }],
                                                        valid: true,
                                                    }],
                                                    valid: true,
                                                },
                                                JfrField {
                                                    idx: 21,
                                                    is_array: false,
                                                    name: "modifiers".to_string(),
                                                    type_name: "int".to_string(),
                                                    tick_unit: None,
                                                    unit: None,
                                                    children: vec![],
                                                    valid: true,
                                                },
                                            ],
                                            valid: true,
                                        },
                                        JfrField {
                                            idx: 22,
                                            is_array: false,
                                            name: "name".to_string(),
                                            type_name: "jdk.types.Symbol".to_string(),
                                            tick_unit: None,
                                            unit: None,
                                            children: vec![JfrField {
                                                idx: 23,
                                                is_array: false,
                                                name: "string".to_string(),
                                                type_name: "java.lang.String".to_string(),
                                                tick_unit: None,
                                                unit: None,
                                                children: vec![],
                                                valid: true,
                                            }],
                                            valid: true,
                                        },
                                        JfrField {
                                            idx: 24,
                                            is_array: false,
                                            name: "descriptor".to_string(),
                                            type_name: "jdk.types.Symbol".to_string(),
                                            tick_unit: None,
                                            unit: None,
                                            children: vec![JfrField {
                                                idx: 25,
                                                is_array: false,
                                                name: "string".to_string(),
                                                type_name: "java.lang.String".to_string(),
                                                tick_unit: None,
                                                unit: None,
                                                children: vec![],
                                                valid: true,
                                            }],
                                            valid: true,
                                        },
                                        JfrField {
                                            idx: 26,
                                            is_array: false,
                                            name: "modifiers".to_string(),
                                            type_name: "int".to_string(),
                                            tick_unit: None,
                                            unit: None,
                                            children: vec![],
                                            valid: true,
                                        },
                                        JfrField {
                                            idx: 27,
                                            is_array: false,
                                            name: "hidden".to_string(),
                                            type_name: "boolean".to_string(),
                                            tick_unit: None,
                                            unit: None,
                                            children: vec![],
                                            valid: true,
                                        },
                                    ],
                                    valid: true,
                                },
                                JfrField {
                                    idx: 28,
                                    is_array: false,
                                    name: "lineNumber".to_string(),
                                    type_name: "int".to_string(),
                                    tick_unit: None,
                                    unit: None,
                                    children: vec![],
                                    valid: true,
                                },
                                JfrField {
                                    idx: 29,
                                    is_array: false,
                                    name: "bytecodeIndex".to_string(),
                                    type_name: "int".to_string(),
                                    tick_unit: None,
                                    unit: None,
                                    children: vec![],
                                    valid: true,
                                },
                                JfrField {
                                    idx: 30,
                                    is_array: false,
                                    name: "type".to_string(),
                                    type_name: "jdk.types.FrameType".to_string(),
                                    tick_unit: None,
                                    unit: None,
                                    children: vec![JfrField {
                                        idx: 31,
                                        is_array: false,
                                        name: "description".to_string(),
                                        type_name: "java.lang.String".to_string(),
                                        tick_unit: None,
                                        unit: None,
                                        children: vec![],
                                        valid: true,
                                    }],
                                    valid: true,
                                },
                            ],
                            valid: true,
                        },
                    ],
                    valid: true,
                },
                JfrField {
                    idx: 32,
                    is_array: false,
                    name: "state".to_string(),
                    type_name: "jdk.types.ThreadState".to_string(),
                    tick_unit: None,
                    unit: None,
                    children: vec![JfrField {
                        idx: 33,
                        is_array: false,
                        name: "name".to_string(),
                        type_name: "java.lang.String".to_string(),
                        tick_unit: None,
                        unit: None,
                        children: vec![],
                        valid: true,
                    }],
                    valid: true,
                },
            ],
            valid: true,
        };

        assert_eq!(root, expected);
        assert_eq!(count, 34);
    }
}
