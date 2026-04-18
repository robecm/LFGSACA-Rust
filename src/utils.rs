pub const MSB: usize = 1_usize << (std::mem::size_of::<usize>() * 8 - 1);
pub const UMASK: usize = !MSB;

#[inline(always)]
pub fn is_marked(v: usize) -> bool {
    (v & MSB) != 0
}

#[inline(always)]
pub fn mark(v: usize) -> usize {
    v | MSB
}

#[inline(always)]
pub fn unmark(v: usize) -> usize {
    v & UMASK
}
