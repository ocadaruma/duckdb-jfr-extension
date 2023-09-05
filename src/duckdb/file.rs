use std::ffi::CString;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use crate::duckdb::bindings::{duckdb_client_context, duckdb_file_get_size, duckdb_file_handle, duckdb_file_read, duckdb_file_seek, duckdb_open_file, FILE_FLAGS_READ};

pub struct FileHandle {
    ptr: duckdb_file_handle,
    size: u64,
    pos: u64,
}

impl FileHandle {
    pub fn open(context: duckdb_client_context, path: &str) -> Self {
        let path = CString::new(path).expect("CString::new failed");
        let ptr = unsafe { duckdb_open_file(context, path.as_ptr(), FILE_FLAGS_READ) };
        let size = unsafe { duckdb_file_get_size(ptr) };

        Self {
            ptr,
            size: size as u64,
            pos: 0,
        }
    }
}

impl Read for FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = unsafe {
            duckdb_file_read(self.ptr, buf.as_mut_ptr().cast(), buf.len() as i64)
        };
        if read < 0 {
            Err(std::io::Error::from(ErrorKind::Other))
        } else {
            self.pos += read as u64;
            Ok(read as usize)
        }
    }
}

impl Seek for FileHandle {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::End(pos) => (self.size as i64 + pos) as u64,
            SeekFrom::Current(pos) => (self.pos as i64 + pos) as u64,
        };
        unsafe { duckdb_file_seek(self.ptr, new_pos) };
        Ok(new_pos)
    }
}
