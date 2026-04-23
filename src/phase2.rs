use crate::utils::{is_marked, mark, unmark, UMASK};

pub fn phase2(sa: &mut [usize], mut gstarts: Vec<usize>, isa_prev: &[usize], n: usize) {
    sa.fill(UMASK);

    for i in 0..n {
        let isa = isa_prev[2 * i];
        if is_marked(isa) {
            sa[unmark(isa)] = mark(i);
        }
    }
    walk(sa, &mut gstarts, isa_prev, n);
}

pub fn phase2_circular(sa: &mut [usize], mut gstarts: Vec<usize>, isa_prev: &[usize], n: usize) {
    sa.fill(UMASK);

    for i in 0..n {
        let isa = isa_prev[2 * i];
        if is_marked(isa) {
            let pos = unmark(isa);
            sa[pos] = mark(i);
        }
    }
    walk(sa, &mut gstarts, isa_prev, n);
}

fn walk(sa: &mut [usize], gstarts: &mut [usize], isa_prev: &[usize], n: usize) {
    let walk_logic = |mut curr: usize, sa_ref: &mut [usize], gs: &mut [usize]| {
        loop {
            let isa = isa_prev[2 * curr];
            if is_marked(isa) { break; }

            let p = isa_prev[2 * curr + 1];
            let target_grp = isa;
            let sr = gs[target_grp];
            gs[target_grp] += 1;

            let up = unmark(p);
            let msb = (up + 1 < curr) || (up > curr);

            sa_ref[sr] = if msb { mark(curr) } else { curr };

            if !is_marked(p) { break; }
            curr = up;
        }
    };

    walk_logic(n - 1, sa, gstarts);

    for i in 0..n {
        let sv = sa[i];
        if sv != UMASK && is_marked(sv) {
            sa[i] = unmark(sv);
            if sa[i] > 0 { walk_logic(sa[i] - 1, sa, gstarts); }
        }
    }

    for j in 0..n {
        sa[j] = unmark(sa[j]);
    }
}