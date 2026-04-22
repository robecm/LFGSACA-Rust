pub mod phase1;
pub mod phase2;
pub mod pss;
pub mod utils;

use crate::phase1::{build_c, insert_leaves, phase1, write_group_sizes};
use crate::phase2::{phase2, phase2_circular};
use crate::pss::{compute_ls_types, compute_pss, compute_lpss};

pub enum FgsacaMode { SuffixArray, BBWT, EBWT }

pub fn fgsaca(text: &[u8], mode: FgsacaMode) -> Vec<usize> {
    let n = text.len();
    if n == 0 { return Vec::new(); }

    let pss = match mode {
        FgsacaMode::EBWT => compute_lpss(text, n),
        _ => compute_pss(text, n),
    };

    let types = compute_ls_types(text, n);
    let c = build_c(text, n, 256, &types);

    let mut sa = vec![0; n];
    let mut isa = vec![0; n];

    write_group_sizes(&mut sa, &c, 256);
    insert_leaves(text, n, &mut sa, &mut isa, &c);

    let gstarts = phase1(&mut sa, &pss, &mut isa, n);

    let mut isa_prev = vec![0; 2 * n];
    for i in 0..n {
        isa_prev[2 * i] = isa[i];
        isa_prev[2 * i + 1] = if i == 0 { 0 } else { pss[i] };
    }

    drop(isa);
    drop(pss);
    drop(types);

    match mode {
        FgsacaMode::SuffixArray => phase2(&mut sa, gstarts, &isa_prev, n),
        _ => phase2_circular(&mut sa, gstarts, &isa_prev, n),
    }

    sa
}