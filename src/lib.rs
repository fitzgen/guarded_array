extern crate memmap;

use std::io;
use std::marker::PhantomData;
use std::ops;
use std::ptr;
use std::slice;

mod ffi {
    use std::os::raw;

    extern "C" {
        pub fn mprotect(addr: *mut raw::c_void, len: usize, prot: raw::c_int) -> raw::c_int;
    }

    pub const PROT_NONE: raw::c_int = 0;
}

const PAGE_SIZE: usize = 0x1000;

struct Mapping {
    buf: memmap::Mmap,
}

impl Mapping {
    fn with_capacity(capacity: usize) -> io::Result<Mapping> {
        assert!(capacity > 0);
        assert_eq!(capacity % PAGE_SIZE, 0);

        let mut buf = memmap::Mmap::anonymous(capacity + PAGE_SIZE, memmap::Protection::ReadWrite)?;
        assert_eq!(buf.ptr() as usize % PAGE_SIZE, 0);

        unsafe {
            let guard_ptr = buf.mut_ptr().offset(capacity as isize);
            if ffi::mprotect(guard_ptr as *mut _, PAGE_SIZE, ffi::PROT_NONE) != 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "mprotect failed"));
            }
        }

        Ok(Mapping { buf })
    }
}

/// TODO FITZGEN
pub struct GuardedArray<T> {
    mapping: Mapping,
    len: usize,
    _phantom: PhantomData<*mut T>,
}

impl<T> ops::Deref for GuardedArray<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.as_ptr(), self.len)
        }
    }
}

impl<T> ops::DerefMut for GuardedArray<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.as_mut_ptr(), self.len)
        }
    }
}

impl<T> GuardedArray<T> {
    /// TODO FITZGEN
    pub fn with_capacity(capacity: usize) -> io::Result<GuardedArray<T>> {
        assert!(capacity > 0);

        let mut capacity = capacity;
        if capacity % PAGE_SIZE > 0 {
            capacity += PAGE_SIZE - capacity % PAGE_SIZE;
        }

        Ok(GuardedArray {
            mapping: Mapping::with_capacity(capacity)?,
            len: 0,
            _phantom: PhantomData,
        })
    }

    /// TODO FITZGEN
    pub fn as_ptr(&self) -> *const T {
        self.mapping.buf.ptr() as *const T
    }

    /// TODO FITZGEN
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.mapping.buf.mut_ptr() as *mut T
    }

    /// TODO FITZGEN
    pub fn push(&mut self, value: T) {
        unsafe {
            let end = self.as_mut_ptr().offset(self.len as isize);
            ptr::write(end, value);
            self.len += 1;
        }
    }

    /// TODO FITZGEN
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(ptr::read(self.get_unchecked(self.len())))
            }
        }
    }

    /// TODO FITZGEN
    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len();
        assert!(index < len);
        unsafe {
            // infallible
            let ret;
            {
                // the place we are taking from.
                let ptr = self.as_mut_ptr().offset(index as isize);
                // copy it out, unsafely having a copy of the value on
                // the stack and in the vector at the same time.
                ret = ptr::read(ptr);

                // Shift everything down to fill in that spot.
                ptr::copy(ptr.offset(1), ptr, len - index - 1);
            }
            self.len -= 1;
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
