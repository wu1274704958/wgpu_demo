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

