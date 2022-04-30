use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

use hyper::body::{Bytes, HttpBody};
use hyper::Body;
use tokio::runtime::{Builder, Runtime};
use wasmer_wasi::{WasiFile, WasiFsError};


/// for impl the interface WasiFile
macro_rules! impl_wasi_file {
    ($name:ident) => {
        impl WasiFile for $name {
            fn last_accessed(&self) -> u64 {
                0
            }

            fn last_modified(&self) -> u64 {
                0
            }

            fn created_time(&self) -> u64 {
                0
            }

            fn size(&self) -> u64 {
                0
            }

            fn set_len(&mut self, _new_size: u64) -> std::result::Result<(), WasiFsError> {
                Err(WasiFsError::PermissionDenied)
            }

            fn unlink(&mut self) -> std::result::Result<(), WasiFsError> {
                Ok(())
            }

            fn bytes_available(&self) -> std::result::Result<usize, WasiFsError> {
                Ok(self.bytes_available())
            }
        }
    };
}


/// for impl the interface Seek which can not seek
macro_rules! impl_not_seek {
    ($name: ident) => {
        impl Seek for $name {
            fn seek(&mut self, _pos: SeekFrom) -> Result<u64> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not seek ", stringify!($name)),
                ))
            }
        }
    };
}


/// for impl the interface Seek which can not read
macro_rules! impl_unreadable {
    ($name: ident) => {
        impl Read for $name {
            fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not read from stdout", stringify!($name)),
                ))
            }

            fn read_to_end(&mut self, _buf: &mut Vec<u8>) -> Result<usize> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not read from stdout", stringify!($name)),
                ))
            }

            fn read_to_string(&mut self, _buf: &mut String) -> Result<usize> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not read from stdout", stringify!($name)),
                ))
            }

            fn read_exact(&mut self, _buf: &mut [u8]) -> Result<()> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not read from stdout", stringify!($name)),
                ))
            }
        }
    };
}


/// for impl the interface Write which can not write
macro_rules! impl_unwritable {
    ($name:ident) => {
        impl Write for $name {
            fn write(&mut self, _buf: &[u8]) -> Result<usize> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not write to ", stringify!($name)),
                ))
            }

            fn flush(&mut self) -> Result<()> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not write to ", stringify!($name)),
                ))
            }

            fn write_all(&mut self, _buf: &[u8]) -> Result<()> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not write to ", stringify!($name)),
                ))
            }

            fn write_fmt(&mut self, _fmt: std::fmt::Arguments) -> Result<()> {
                Err(Error::new(
                    ErrorKind::Other,
                    concat!("can not write to ", stringify!($name)),
                ))
            }
        }
    };
}


/// redirect the request body to stdin
#[derive(Debug)]
pub(super) struct Stdin {
    /// the buffer array
    _buffer: Bytes,
    /// the http body data
    _http_body: Body,
    /// async data handle
    _async_handle: Runtime,
}


impl Stdin {
    pub(super) fn new(body: Body) -> Result<Self> {
        Ok(Self {
            _buffer: Bytes::new(),
            _http_body: body,
            _async_handle: Builder::new_current_thread().enable_all().build()?,
        })
    }

    /// poll the new chunk to this buffer (replace the old buffer)
    #[inline(always)]
    fn poll_data(&mut self) -> Result<bool> {
        match self._async_handle.block_on(self._http_body.data()) {
            Some(Ok(chunk)) => {
                self._buffer = chunk;
                Ok(true)
            }
            Some(Err(e)) => Err(Error::new(ErrorKind::Other, e.to_string())),
            None => Ok(false),
        }
    }

    #[inline(always)]
    fn bytes_available(&self) -> usize {
        self._http_body.size_hint().lower() as usize
    }
}


impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut size = 0;

        if !self._buffer.is_empty() {
            if self._buffer.len() >= buf.len() {
                buf.copy_from_slice(&self._buffer.as_ref()[..buf.len()]);
                self._buffer = self._buffer.slice(buf.len()..);
                return Ok(buf.len());
            } else {
                buf[..self._buffer.len()].copy_from_slice(&self._buffer.as_ref());
                size = self._buffer.len();
            }
        }

        while self.poll_data()? && !self._buffer.is_empty() {
            let remain = buf.len() - size;
            if self._buffer.len() >= remain {
                buf[size..].copy_from_slice(&self._buffer.as_ref()[..remain]);
                self._buffer = self._buffer.slice(remain..);
                return Ok(buf.len());
            } else {
                buf[size..size + self._buffer.len()].copy_from_slice(&self._buffer.as_ref());
                size += self._buffer.len();
                self._buffer = self._buffer.slice(self._buffer.len()..);
            }
        }

        Ok(size)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut size = 0;

        // copy the now chunk to buf
        if !self._buffer.is_empty() {
            size += self._buffer.len();
            buf.extend(&self._buffer);
        }

        // poll new data until EOF
        while self.poll_data()? {
            size += self._buffer.len();
            buf.extend(&self._buffer);
        }

        Ok(size)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let old_len = buf.len();
        let res = self.read_to_end(unsafe { buf.as_mut_vec() });

        if res.is_err() {
            return res;
        }

        // check if the data valid
        match std::str::from_utf8((&buf[old_len..]).as_ref()).is_ok() {
            true => res,
            false => Err(Error::new(
                ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            )),
        }
    }
}


// the Stdin only can read
impl_wasi_file!(Stdin);
impl_not_seek!(Stdin);
impl_unwritable!(Stdin);


/// stdout for wasm function, just buffer it into vector
#[derive(Debug, Clone)]
pub(super) struct Stdout {
    _buffer: Vec<u8>,
}


impl Stdout {
    pub(super) fn new() -> Self {
        Self {
            _buffer: Vec::new(),
        }
    }

    /// take the buffer data with zero copy
    pub(super) fn take_buffer(&mut self) -> Vec<u8> {
        std::mem::take(&mut self._buffer)
    }


    #[inline(always)]
    fn bytes_available(&self) -> usize {
        0
    }
}


impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self._buffer.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self._buffer.extend(buf);
        Ok(())
    }
}


// the Stdout only can write
impl_wasi_file!(Stdout);
impl_not_seek!(Stdout);
impl_unreadable!(Stdout);



/// redirect stderr to watchdog log
#[derive(Debug)]
pub(super) struct Stderr {
    _logger_name: String,
    _buffer: Vec<u8>,
    _log_prefix: bool,
    _buf_max_size: usize,
}


impl Stderr {
    pub(in super) fn new(logger_name: String, log_prefix: bool, log_buf_size: usize) -> Self {
        Self {
            _logger_name: logger_name,
            _buffer: Vec::new(),
            _log_prefix: log_prefix,
            _buf_max_size: log_buf_size,
        }
    }

    #[inline(always)]
    fn bytes_available(&self) -> usize {
        0
    }

    fn flush_inner(&mut self) -> Result<()> {
        if !self._buffer.is_empty() {
            let str = match std::str::from_utf8(self._buffer.as_slice()) {
                Ok(s) => s,
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    ));
                }
            };

            if self._log_prefix {
                str.split('\n').for_each(|s| {
                    if !s.is_empty() {
                        eprintln!("[watchdog function] {}: {}", self._logger_name, s);
                    }
                });
            } else {
                eprint!("{}", str);
            }
            self._buffer.clear();
        }
        Ok(())
    }
}


/// bind to the log
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self._buffer.extend(buf);
        if self._buffer.len() >= self._buf_max_size {
            self.flush_inner()?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self._buffer.extend(buf);
        if self._buffer.len() >= self._buf_max_size {
            return self.flush_inner();
        }
        Ok(())
    }
}


impl Drop for Stderr {
    /// flush the buffer to logger
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.flush_inner();
    }
}


// the Stderr only can write
impl_wasi_file!(Stderr);
impl_unreadable!(Stderr);
impl_not_seek!(Stderr);
