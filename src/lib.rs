pub mod phase1;
pub mod phase2;
pub mod pss;
pub mod utils;

use crate::phase1::{build_c, insert_leaves, phase1, write_group_sizes};
use crate::phase2::phase2;
use crate::pss::{compute_ls_types, compute_pss};
use crate::utils::{is_marked, unmark};

pub fn fgsaca(text: &[u8]) -> Vec<usize> {
    let n = text.len();
    if n == 0 {
        return Vec::new();
    }

    let debug = n < 20; // Solo imprime en pruebas cortas
    if debug {
        println!(
            "\n[DEBUG] --- STARTING FGSACA FOR: {:?} ---",
            String::from_utf8_lossy(text)
        );
    }

    let mut pss = compute_pss(text, n);
    let types = compute_ls_types(text, n);
    let c = build_c(text, n, 256, &types);

    let mut sa = vec![0; n];
    let mut isa = vec![0; n];

    write_group_sizes(&mut sa, &c, 256);
    insert_leaves(text, n, &mut sa, &mut isa, &c);

    let gstarts = phase1(&mut sa, &pss, &mut isa, n);

    if debug {
        println!("[DEBUG] --- HANDOFF TO PHASE 2 ---");
        println!("[DEBUG] SA Array : {:?}", sa);

        // Imprime el ISA poniendo un asterisco '*' si tiene la marca MSB encendida
        let isa_str: Vec<String> = isa
            .iter()
            .map(|&x| {
                if is_marked(x) {
                    format!("*{}", unmark(x))
                } else {
                    unmark(x).to_string()
                }
            })
            .collect();
        println!("[DEBUG] ISA Array: {:?}", isa_str);
        println!("[DEBUG] GSTARTS  : {:?}", gstarts);
    }

    let mut isa_prev = vec![0; 2 * n];
    for i in 0..n {
        isa_prev[2 * i] = isa[i];
        isa_prev[2 * i + 1] = if i == 0 { 0 } else { pss[i] };
    }

    drop(isa);
    drop(pss);
    drop(types);

    phase2(&mut sa, gstarts, &isa_prev, n);
    sa
}
