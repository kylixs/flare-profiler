use super::native::MutByteArray;

pub struct MemoryAllocation {
    pub ptr: MutByteArray,
    pub len: usize
}
