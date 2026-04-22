use crate::utils::{is_marked, mark, unmark};
use std::io::{self, Write};

// ═══════════════════════════════════════════════════════════════════════════
//  DEBUG FLAG — set to `false` for production builds
// ═══════════════════════════════════════════════════════════════════════════
const DEBUG: bool = true;

const MSB: usize = 1_usize << 63;

/// Panics with rich context when `idx` is out-of-bounds or MSB-set.
#[cold]
#[inline(never)]
fn bad_index(idx: usize, len: usize, site: &'static str) -> ! {
    panic!(
        "\n[PHASE1 BAD INDEX]\n  site  : {site}\n  idx   : {idx:#018x} = {idx}\n  len   : {len}\n  MSB?  : {}",
        idx & MSB != 0
    );
}

#[inline(always)]
fn chk(idx: usize, len: usize, site: &'static str) -> usize {
    if DEBUG && (idx >= len || idx & MSB != 0) {
        bad_index(idx, len, site);
    }
    idx
}

// ───────────────────────────────────────────────────────────────────────────

pub fn build_c(s: &[u8], n: usize, sigma: usize, types: &[u8]) -> Vec<usize> {
    let mut c = vec![0; 2 * sigma + 2];
    for i in 0..n {
        c[2 * (s[i] as usize) + (types[i] as usize)] += 1;
    }
    let mut j = 0;
    for i in 0..=(2 * sigma) {
        let old = c[i];
        c[i] = j;
        j += old;
    }
    c[2 * sigma + 1] = j;
    c
}

pub fn write_group_sizes(sa: &mut [usize], c: &[usize], sigma: usize) {
    for i in 0..(2 * sigma) {
        if c[i + 1] > c[i] {
            sa[c[i]] = c[i + 1] - c[i];
        }
    }
}

pub fn insert_leaves(s: &[u8], n: usize, sa: &mut [usize], isa: &mut [usize], c: &[usize]) {
    let mut c1 = s[n - 1];
    let mut t = 0;

    for i in (0..n).rev() {
        let c0 = s[i];
        if i < n - 1 {
            t = if (c0 as u16) < (c1 as u16) + t { 1 } else { 0 };
        }
        c1 = c0;
        let cc = (c0 as usize) * 2 + t as usize;
        let gstart = c[cc];

        isa[i] = gstart;

        if t == 0 {
            sa[gstart] = sa[gstart].wrapping_sub(1);
            let pos = gstart.wrapping_add(sa[gstart]);
            sa[pos] = mark(i);
        }
    }
}

pub fn phase1(sa: &mut [usize], pss: &[usize], isa: &mut [usize], n: usize) -> Vec<usize> {
    let mut gstarts = Vec::new();
    let mut gend = (n as isize) - 1;
    let update_interval = (n / 100).max(1) as isize;

    while gend >= 0 {
        if (n as isize - gend) % update_interval == 0 {
            print!("\r[INFO] Phase I Progress: {}%  ", (((n as isize - gend) as f64 / n as f64) * 100.0) as usize);
            io::stdout().flush().unwrap();
        }

        let sv = sa[gend as usize];
        if !is_marked(sv) {
            gend -= 1;
            continue;
        }

        let s_val = unmark(sv);
        let gstart_raw = isa[s_val];

        if is_marked(gstart_raw) {
            gend -= 1;
            continue;
        }

        let gstart = gstart_raw;
        let mut num = (gend as usize) + 1 - gstart;

        // ── Singleton path ───────────────────────────────────────────────────
        if num == 1 {
            isa[s_val] = mark(gstart);
            let p_raw = pss[s_val];
            let p = unmark(p_raw);
            if p < n {
                let gs_raw = isa[p];
                if !is_marked(gs_raw) {
                    let gs = chk(gs_raw, n, "singleton: gs=isa[p] (unmarked)");
                    let sa_gs = sa[gs];

                    // ── Diagnostic print for every singleton transition ───────
                    if DEBUG {
                        eprintln!(
                            "[SING] s={s_val} -> p={p} lc={} gs={gs} sa[gs]={sa_gs}",
                            is_marked(p_raw)
                        );
                    }

                    if sa_gs == 1 && is_marked(p_raw) {
                        // Last slot and last child: group becomes a singleton.
                        sa[gs] = mark(p);
                        isa[p] = mark(gs);
                    } else if sa_gs == 1 && !is_marked(p_raw) {
                        // ── BUG SITE A ────────────────────────────────────────
                        // Last slot, but p is NOT the last child.
                        // The original code fell through to `else if sa_gs != 1`
                        // (false) and did NOTHING — silently losing the element.
                        // We print and recover: place p in the last slot (gs+0=gs).
                        if DEBUG {
                            eprintln!(
                                "[BUG-A] sa[gs]=1 but p is NOT last child. \
                                 s={s_val} p={p} gs={gs} — was silently dropped!"
                            );
                        }
                        sa[gs] = p;        // non-last-child → not marked
                        isa[p] = mark(gs); // concrete position known
                    } else if sa_gs == 0 {
                        // ── BUG SITE B ────────────────────────────────────────
                        // Counter is already zero — wrapping_sub would give
                        // usize::MAX, corrupting everything downstream.
                        if DEBUG {
                            eprintln!(
                                "[BUG-B] sa[gs]=0 counter already exhausted! \
                                 s={s_val} p={p} gs={gs} p_raw={p_raw:#018x}"
                            );
                        }
                        // Emergency recovery: use gs as the slot.
                        sa[gs] = if is_marked(p_raw) { mark(p) } else { p };
                        isa[p] = mark(gs);
                    } else {
                        // Normal path: sa_gs > 1.
                        let new_sa_gs = sa_gs - 1; // safe: sa_gs > 1 > 0
                        sa[gs] = new_sa_gs;
                        let pos = gs + new_sa_gs;  // safe addition
                        chk(pos, n, "singleton: pos=gs+new_sa_gs");
                        sa[pos] = if is_marked(p_raw) { mark(p) } else { p };
                        isa[p] = mark(pos);
                    }
                } else {
                    // isa[p] already marked → concrete position is known.
                    let pos = chk(unmark(gs_raw), n, "singleton: pos=unmark(isa[p]) marked");
                    sa[pos] = if is_marked(p_raw) { mark(p) } else { p };
                    isa[p] = mark(pos);
                }
            }
            gend -= 1;
            continue;
        }

        // ── Multi-element group ──────────────────────────────────────────────
        gstarts.push(gstart);
        let mut num_factors = 0;

        while num_factors < num {
            let item = unmark(sa[gstart + num_factors]);
            if unmark(pss[item]) == n {
                isa[item] = mark(gstart + num - 1 - num_factors);
                num_factors += 1;
            } else {
                break;
            }
        }

        if num_factors > 0 {
            for i in num_factors..num {
                let item = unmark(sa[gstart + i]);
                sa[gstart + i - num_factors] = if is_marked(pss[item]) { mark(item) } else { item };
            }
            num -= num_factors;
            gend -= num_factors as isize;
            if num == 0 {
                gend = (gstart as isize) - 1;
                continue;
            }
        } else {
            for i in 0..num {
                let item = unmark(sa[gstart + i]);
                sa[gstart + i] = if is_marked(pss[item]) { mark(item) } else { item };
            }
        }

        let mut elements = vec![0; num];
        for i in 0..num { elements[i] = sa[gstart + i]; }
        elements.sort_unstable_by_key(|&x| unmark(pss[unmark(x)]));

        let mut singles_lc  = Vec::new();
        let mut singles_nlc = Vec::new();
        let mut non_singles: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;

        while i < num {
            let val = elements[i];
            let p = unmark(pss[unmark(val)]);
            let mut cnt = 0;
            let mut is_lc = false;
            while i + cnt < num && unmark(pss[unmark(elements[i + cnt])]) == p {
                if is_marked(elements[i + cnt]) { is_lc = true; }
                cnt += 1;
            }
            if cnt == 1 {
                if is_lc { singles_lc.push(elements[i]); }
                else      { singles_nlc.push(elements[i]); }
            } else {
                let key = (cnt - 1) * 2 + (if is_lc { 0 } else { 1 });
                for k in 0..cnt { non_singles.push((elements[i + k], key)); }
            }
            i += cnt;
        }

        non_singles.sort_unstable_by_key(|x| x.1);

        // Write back sorted children into sa[gstart..]
        let mut idx = 0;
        for &val in &singles_lc {
            let uval = unmark(val);
            sa[gstart + idx] = if is_marked(pss[uval]) { mark(uval) } else { uval };
            idx += 1;
        }
        for &val in &singles_nlc {
            let uval = unmark(val);
            sa[gstart + idx] = if is_marked(pss[uval]) { mark(uval) } else { uval };
            idx += 1;
        }
        for &(val, _) in &non_singles {
            let uval = unmark(val);
            sa[gstart + idx] = if is_marked(pss[uval]) { mark(uval) } else { uval };
            idx += 1;
        }

        // Build bucket list
        let mut buckets: Vec<(isize, usize, usize)> = Vec::new();
        let mut cur = 0usize;
        if !singles_lc.is_empty()  { buckets.push((0,  0,   singles_lc.len())); }
        cur += singles_lc.len();
        if !singles_nlc.is_empty() { buckets.push((1, cur, cur + singles_nlc.len())); }
        cur += singles_nlc.len();

        let mut prev_key = -1isize;
        let mut bstart = cur;
        for &(_, key) in &non_singles {
            if (key as isize) != prev_key {
                if prev_key >= 0 { buckets.push((prev_key, bstart, cur)); }
                bstart = cur;
                prev_key = key as isize;
            }
            cur += 1;
        }
        if !non_singles.is_empty() { buckets.push((prev_key, bstart, cur)); }
        buckets.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        for &(key, bs, bend) in &buckets {
            let is_final = key % 2 == 0;

            if is_final {
                // ── is_final: each element's parent slot is decremented ───────
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];

                    if is_marked(p_raw) {
                        let pos = chk(unmark(p_raw), n, "is_final: pos=unmark(isa[s])");
                        sa[pos] = mark(s);
                        isa[s] = mark(pos);
                    } else {
                        let p = chk(p_raw, n, "is_final: p=isa[s]");
                        let sa_p = sa[p];

                        // ── Diagnostic ───────────────────────────────────────
                        if DEBUG && sa_p == 0 {
                            eprintln!(
                                "[BUG-C is_final] sa[p]=0 before decrement! \
                                 s={s} p={p} key={key} gstart={gstart} bs={bs} bend={bend}"
                            );
                        }

                        let new_sa_p = sa_p.wrapping_sub(1);
                        sa[p] = new_sa_p;
                        let pos = p.wrapping_add(new_sa_p);
                        chk(pos, n, "is_final: pos=p.wrapping_add(sa[p]-1)");
                        sa[pos] = mark(s);
                        isa[s] = mark(pos);
                    }
                }
            } else {
                // ── non-final: three-pass insertion into parent group ─────────

                // Pass 1 — reserve slots by decrementing sa[p] per child.
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];
                    if !is_marked(p_raw) {
                        let p = chk(p_raw, n, "non-final pass1: p=isa[s]");

                        // ── Diagnostic ───────────────────────────────────────
                        if DEBUG && sa[p] == 0 {
                            eprintln!(
                                "[BUG-D non-final pass1] sa[p]=0 before wrapping_sub! \
                                 s={s} p={p} key={key} gstart={gstart} bs={bs} bend={bend}"
                            );
                        }

                        sa[p] = sa[p].wrapping_sub(1);
                    }
                }

                // Pass 2 — compute base pointer; write children.
                let mut prev_p = usize::MAX;
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];

                    if is_marked(p_raw) {
                        let pos = chk(unmark(p_raw), n, "non-final pass2: pos=unmark(isa[s])");
                        sa[pos] = s;
                        isa[s] = mark(pos);
                    } else {
                        let p = chk(p_raw, n, "non-final pass2: p=isa[s]");
                        let new_start = p.wrapping_add(sa[p]);
                        chk(new_start, n, "non-final pass2: new_start=p.wrapping_add(sa[p])");
                        isa[s] = new_start; // store plain (unmarked) for Pass 3

                        if p != prev_p {
                            sa[new_start] = 0;
                            prev_p = p;
                        }
                    }
                }

                // Pass 3 — fill counter: increment sa[new_start] per child.
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];
                    if !is_marked(p_raw) {
                        chk(p_raw, n, "non-final pass3: p_raw=isa[s] (new_start)");
                        sa[p_raw] = sa[p_raw].wrapping_add(1);
                    }
                }
            }
        }
        gend = (gstart as isize) - 1;
    }

    for (i, &gs) in gstarts.iter().enumerate() {
        sa[gs] = i;
    }
    for i in 0..n {
        let p_isa = isa[i];
        if !is_marked(p_isa) { isa[i] = sa[p_isa]; }
    }

    println!("\r[INFO] Phase I Progress: 100%      ");
    gstarts
}