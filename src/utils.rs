pub const MSB: u32 = 1 << 31;
pub const UMASK: u32 = !MSB;

#[inline(always)]
pub fn is_marked(v: u32) -> bool {
    (v & MSB) != 0
}

#[inline(always)]
pub fn mark(v: u32) -> u32 {
    v | MSB
}

#[inline(always)]
pub fn unmark(v: u32) -> u32 {
    v & UMASK
}