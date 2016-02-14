use std::marker::PhantomData;
use std::mem;
use std::ptr;


/// Unsafe wrapper around a trait object.
///
/// This struct can be used to move around a trait object when its lifetime is not expressible. Be
/// aware though that moving the object from which the trait has been built will cause crashes,
/// since the pointer stored in this struct will not change with it. For this reason, getting the
/// trait back from this object is unsafe.
pub struct UnsafeTraitObject<T: ?Sized> {
    // TODO: Unfortunately we cannot use a fixed size array here, because
    //       [u8; mem::size_of::<&T>()] is illegal in Rust for now (Rust 1.7)
    //       when this will be available, we can avoid double indirection and store in this struct
    //       the array inline. Be aware of alignment issues though!
    raw_ptr: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized> UnsafeTraitObject<T> {
    pub fn new(e: &T) -> Self {
        let mut r = vec![0u8; mem::size_of::<&T>()];
        unsafe {
            let ptr: *mut &T = mem::transmute(r.as_mut_ptr());
            ptr::write(ptr, e);
        };
        UnsafeTraitObject {
            raw_ptr: r,
            _phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub unsafe fn as_inner(&self) -> &T {
        let ptr: *mut &T = mem::transmute(self.raw_ptr.as_ptr());
        ptr::read(ptr)
    }

    pub unsafe fn as_inner_mut(&mut self) -> &mut T {
        let ptr: *mut &mut T = mem::transmute(self.raw_ptr.as_ptr());
        ptr::read(ptr)
    }
}

unsafe impl<'a, T: ?Sized + 'a> Send for UnsafeTraitObject<T> where &'a T: Send
{}
