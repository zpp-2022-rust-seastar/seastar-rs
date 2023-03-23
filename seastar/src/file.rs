use crate::assert_runtime_is_running;
use cxx::UniquePtr;
use ffi::*;
use std::alloc::{self, Layout};
use std::io;
use std::ops::{Deref, Index, IndexMut};
use std::path::Path;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type OpenOptions;

        pub fn get_read(&self) -> bool;
        pub fn get_write(&self) -> bool;
        pub fn get_create(&self) -> bool;
    }

    #[namespace = "seastar_ffi"]
    unsafe extern "C++" {
        type VoidFuture = crate::cxx_async_futures::VoidFuture;
        type IntFuture = crate::cxx_async_futures::IntFuture;
    }

    #[namespace = "seastar_ffi::file"]
    unsafe extern "C++" {
        include!("seastar/src/file.hh");

        type file_t;

        fn open_dma(file: &mut UniquePtr<file_t>, name: &str, opts: &OpenOptions) -> VoidFuture;

        unsafe fn read_dma(
            file: &UniquePtr<file_t>,
            buffer: *mut u8,
            size: u64,
            pos: u64,
        ) -> IntFuture;

        unsafe fn write_dma(
            file: &UniquePtr<file_t>,
            buffer: *mut u8,
            size: u64,
            pos: u64,
        ) -> IntFuture;

        fn flush(file: &UniquePtr<file_t>) -> VoidFuture;

        fn close(file: &UniquePtr<file_t>) -> VoidFuture;

        fn size(file: &UniquePtr<file_t>) -> IntFuture;
    }
}

const ALIGN: usize = 512;
const CHUNK_SIZE: usize = 4096;

/// A buffer that stores/receives data for I/O operations.
/// Its contents are aligned in memory up to 512 bytes.
/// read_dma and write_dma require memory to be aligned.
pub struct DmaBuffer {
    buffer: *mut u8,
    size: usize,
}

impl Deref for DmaBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, ALIGN).unwrap();
        unsafe {
            alloc::dealloc(self.buffer, layout);
        }
    }
}

impl Index<usize> for DmaBuffer {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        self.as_slice().index(idx)
    }
}

impl IndexMut<usize> for DmaBuffer {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.as_mut_slice().index_mut(idx)
    }
}

impl DmaBuffer {
    pub fn from_slice(bytes: &[u8]) -> Self {
        let size = bytes.len();
        assert!(size % CHUNK_SIZE == 0);
        let layout = Layout::from_size_align(size, ALIGN).unwrap();
        unsafe {
            let buffer = alloc::alloc_zeroed(layout);
            let slice = std::slice::from_raw_parts_mut(buffer, size);
            slice.copy_from_slice(bytes);
            Self { buffer, size }
        }
    }

    pub fn copy_from_slice(&mut self, bytes: &[u8]) -> &mut Self {
        self.as_mut_slice().copy_from_slice(bytes);
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.buffer, self.size) }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.buffer, self.size) }
    }
}

/// Interface modelled after std::fs::OpenOptions.
/// It is a builder for the File.
#[derive(Clone)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenOptions {
    /// Creates a new `OpenOptions` struct with no flag set.
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            create: false,
        }
    }

    /// Sets a flag `read` which allows to read from a file.
    pub fn read(&mut self, flag: bool) -> &mut Self {
        self.read = flag;
        self
    }

    /// Sets a flag `write` which allows to write to a file.
    pub fn write(&mut self, flag: bool) -> &mut Self {
        self.write = flag;
        self
    }

    /// Sets a flag `create` which allows to create a new file.
    pub fn create(&mut self, flag: bool) -> &mut Self {
        self.create = flag;
        self
    }

    /// Getter for a `read` flag.
    pub fn get_read(&self) -> bool {
        self.read
    }

    /// Getter for a `write` flag.
    pub fn get_write(&self) -> bool {
        self.write
    }

    /// Getter for a `create` flag.
    pub fn get_create(&self) -> bool {
        self.create
    }

    /// Opens a new file `path` from the OpenOptions set before.
    pub async fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        File::new(&self.clone(), path.as_ref()).await
    }
}

pub struct File {
    inner: UniquePtr<file_t>,
}

impl File {
    /// Creates a new file with `opts` OpenOptions and `path` path to a file.
    ///
    /// Returns a file.
    pub async fn new(opts: &OpenOptions, path: &Path) -> io::Result<File> {
        assert_runtime_is_running();
        let mut f_ptr = UniquePtr::null();
        let name = path.to_str().unwrap();
        let res = open_dma(&mut f_ptr, name, opts).await;
        match res {
            Ok(_) => Ok(File { inner: f_ptr }),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "No read permission")),
        }
    }

    /// Read some bytes at given position.
    ///
    /// Returns the number of bytes read and the original buffer.
    pub async fn read_dma(
        &self,
        buffer: DmaBuffer,
        pos: u64,
    ) -> Result<(usize, DmaBuffer), io::Error> {
        assert_runtime_is_running();
        let size = buffer.size as u64;
        unsafe {
            let fut = read_dma(&self.inner, buffer.buffer, size, pos);
            match fut.await {
                Ok(res) => Ok((res as usize, buffer)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            }
        }
    }

    /// Writes some bytes at given position.
    ///
    /// Returns the number of bytes writted and the original buffer.
    pub async fn write_dma(
        &self,
        buffer: DmaBuffer,
        pos: u64,
    ) -> Result<(usize, DmaBuffer), io::Error> {
        assert_runtime_is_running();
        let size = buffer.size as u64;
        unsafe {
            let fut = write_dma(&self.inner, buffer.buffer, size, pos);
            match fut.await {
                Ok(res) => Ok((res as usize, buffer)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            }
        }
    }

    /// Causes any previously written data to be made stable on presistent storage.
    /// After a flush, data is guaranteed to be on disk.
    pub async fn flush(&self) -> Result<(), io::Error> {
        assert_runtime_is_running();
        match flush(&self.inner).await {
            Ok(_) => Ok(()),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    /// Closes a file.
    pub async fn close(&self) -> Result<(), io::Error> {
        assert_runtime_is_running();
        match close(&self.inner).await {
            Ok(_) => Ok(()),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    /// Returns the number of bytes in a file.
    pub async fn size(&self) -> Result<i32, io::Error> {
        assert_runtime_is_running();
        match size(&self.inner).await {
            Ok(res) => Ok(res),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use crate as seastar;
    use std::{
        io::{Read, Write},
        path::PathBuf,
    };

    fn rand_path() -> PathBuf {
        let fname: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let mut p = std::env::temp_dir();
        p.push(fname);
        p
    }

    #[seastar::test]
    async fn test_file_read_dma() {
        let p = rand_path();
        let msg = b"I <3 seastar!";
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(p.as_path())
            .unwrap()
            .write_all(msg)
            .unwrap();
        let file = OpenOptions::new()
            .read(true)
            .open(p.as_path())
            .await
            .unwrap();
        let buffer = DmaBuffer::from_slice(&[0u8; CHUNK_SIZE]);
        let res = file.read_dma(buffer, 0).await.unwrap();
        file.close().await.unwrap();
        assert_eq!(res.0, msg.len());
        let bytes = &res.1.as_slice()[..msg.len()];
        assert_eq!(bytes, msg);
    }

    #[seastar::test]
    async fn test_file_read_dma_big() {
        let p = rand_path();
        let msg = (0..15000) // less than CHUNK_SIZE * 5
            .map(|_| rand::random::<u8>())
            .collect::<Vec<u8>>();
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(p.as_path())
            .unwrap()
            .write_all(msg.as_ref())
            .unwrap();
        let file = OpenOptions::new()
            .read(true)
            .open(p.as_path())
            .await
            .unwrap();
        let buffer = DmaBuffer::from_slice(&[0u8; CHUNK_SIZE * 5]);
        let res = file.read_dma(buffer, 0).await.unwrap();
        file.close().await.unwrap();
        assert_eq!(res.0, msg.len());
        let bytes = &res.1.as_slice()[..msg.len()];
        assert_eq!(bytes, msg.as_slice());
    }

    #[seastar::test]
    async fn test_file_write_dma() {
        let p = rand_path();
        let mut v = [0u8; CHUNK_SIZE];
        rand::thread_rng().fill(&mut v[..]);
        let buffer = DmaBuffer::from_slice(&v);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(p.as_path())
            .await
            .unwrap();
        let res = file.write_dma(buffer, 0).await.unwrap();
        file.flush().await.unwrap();
        file.close().await.unwrap();
        assert_eq!(res.0, CHUNK_SIZE);
        let mut line = Vec::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(p.as_path())
            .unwrap()
            .read_to_end(&mut line)
            .unwrap();
        let bytes = res.1.as_slice();
        assert_eq!(bytes, line.as_slice());
    }

    #[seastar::test]
    async fn test_file_write_dma_big() {
        let p = rand_path();
        let mut v = [0u8; CHUNK_SIZE * 5];
        rand::thread_rng().fill(&mut v[..]);
        let buffer = DmaBuffer::from_slice(&v);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(p.as_path())
            .await
            .unwrap();
        let res = file.write_dma(buffer, 0).await.unwrap();
        file.flush().await.unwrap();
        file.close().await.unwrap();
        assert_eq!(res.0, CHUNK_SIZE * 5);
        let mut line = Vec::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(p.as_path())
            .unwrap()
            .read_to_end(&mut line)
            .unwrap();
        let bytes = res.1.as_slice();
        assert_eq!(bytes, line.as_slice());
    }

    #[seastar::test]
    async fn test_file_close() {
        let p = rand_path();
        let file = OpenOptions::new()
            .create(true)
            .open(p.as_path())
            .await
            .unwrap();
        file.close().await.unwrap();
    }

    #[seastar::test]
    async fn test_file_size() {
        let p = rand_path();
        let msg = b"I <3 seastar!";
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(p.as_path())
            .unwrap()
            .write_all(msg)
            .unwrap();
        let file = OpenOptions::new().open(p.as_path()).await.unwrap();
        let size = file.size().await.unwrap();
        file.close().await.unwrap();
        assert_eq!(size as usize, msg.len());
    }
}
