use std::mem::size_of;

#[repr(C)]
pub struct FatPtr
{
    ptr: *const u8,
    len: usize
}

impl FatPtr{
    pub fn new<T>(t:&T) -> FatPtr
    {
        FatPtr{
            ptr : unsafe { std::mem::transmute::<*const T, *const u8>(t as *const T) },
            len : std::mem::size_of::<T>()
        }
    }
    pub unsafe fn as_ref_arr(&self) -> &[u8]
    {
        let ptr  = std::mem::transmute::<*const FatPtr,*const u128>(self as *const FatPtr);
        std::mem::transmute::<u128,&[u8]>(*ptr)
    }
}

pub unsafe fn from_raw_parts<T>(t:&T) ->&[u8]
{
    std::slice::from_raw_parts(std::mem::transmute::<_, *const u8>(t), size_of::<T>())
}
pub unsafe fn from_raw_parts_ex<T>(arr:&[T]) ->&[u8]
{
    std::slice::from_raw_parts(std::mem::transmute::<_, *const u8>(&arr[0]), size_of::<T>() * arr.len())
}