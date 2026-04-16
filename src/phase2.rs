use crate::utils::{is_marked, mark, unmark, UMASK};

pub fn phase2(sa: &mut [u32], mut gstarts: Vec<u32>, isa_prev: &[u32], n: usize) {
    let undef = UMASK;
    for i in 0..n { sa[i] = undef; }

    // Pre-poblamos las anclas con su marca MSB vital
    for i in 0..n {
        let isa = isa_prev[2 * i];
        if is_marked(isa) {
            sa[unmark(isa) as usize] = mark(i as u32);
        }
    }

    let mut walk = |mut curr: usize, sa_ref: &mut [u32], gstarts_ref: &mut [u32]| {
        loop {
            let isa = isa_prev[2 * curr];

            // FIX: El ancla ya está colocada y marcada arriba.
            // ¡Solo debemos romper el ciclo sin sobrescribirla!
            if is_marked(isa) { break; }

            let p = isa_prev[2 * curr + 1];
            let isa_idx = isa as usize;

            let sr = gstarts_ref[isa_idx] as usize;
            gstarts_ref[isa_idx] += 1;

            let up = unmark(p) as usize;
            let msb = (up + 1 < curr) || (up > curr);

            sa_ref[sr] = if msb { mark(curr as u32) } else { curr as u32 };

            if !is_marked(p) { break; }
            curr = up;
        }
    };

    walk(n - 1, sa, &mut gstarts);

    for i in 0..n {
        let sv = sa[i];
        if sv != undef && is_marked(sv) {
            sa[i] = unmark(sv);
            // Al no haber borrado la marca del 10, walk(9) por fin se ejecutará aquí.
            if sa[i] > 0 { walk(sa[i] as usize - 1, sa, &mut gstarts); }
        }
    }

    for j in 0..n {
        if sa[j] == undef { sa[j] = 0; } else { sa[j] = unmark(sa[j]); }
    }
}