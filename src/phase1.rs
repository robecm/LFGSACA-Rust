use crate::utils::{is_marked, mark, unmark, MSB, UMASK};
use std::io::{self, Write};

pub fn build_c(s: &[u8], n: usize, sigma: usize, types: &[u8]) -> Vec<u32> {
    let mut c = vec![0; 2 * sigma + 2];
    for i in 0..n { c[2 * (s[i] as usize) + (types[i] as usize)] += 1; }
    let mut j = 0;
    for i in 0..=(2 * sigma) {
        let old = c[i];
        c[i] = j;
        j += old;
    }
    c[2 * sigma + 1] = j;
    c
}

pub fn write_group_sizes(sa: &mut [u32], c: &[u32], sigma: usize) {
    for i in 0..(2 * sigma) {
        if c[i + 1] > c[i] { sa[c[i] as usize] = c[i + 1] - c[i]; }
    }
}

pub fn insert_leaves(s: &[u8], n: usize, sa: &mut [u32], isa: &mut [u32], c: &[u32]) {
    let mut c1 = s[n - 1];
    let mut t = 0;

    for i in (0..n).rev() {
        let c0 = s[i];
        if i < n - 1 { t = if (c0 as u16) < (c1 as u16) + t { 1 } else { 0 }; }
        c1 = c0;
        let cc = (c0 as usize) * 2 + t as usize;
        let gstart = c[cc] as usize;

        isa[i] = gstart as u32;

        if t == 0 {
            sa[gstart] = sa[gstart].wrapping_sub(1);
            let pos = (gstart as u32).wrapping_add(sa[gstart]) as usize;
            sa[pos] = mark(i as u32);
        }
    }
}

pub fn phase1(sa: &mut [u32], pss: &[u32], isa: &mut [u32], n: usize) -> Vec<u32> {
    let mut gstarts = Vec::new();
    let mut gend = (n as isize) - 1;
    let update_interval = (n / 10).max(1) as isize;

    while gend >= 0 {
        if (n as isize - gend) % update_interval == 0 {
            print!("\r[INFO] Phase I Progress: {}%  ", (((n as isize - gend) as f64 / n as f64) * 100.0) as u32);
            io::stdout().flush().unwrap();
        }

        let sv = sa[gend as usize];
        if !is_marked(sv) { gend -= 1; continue; }

        let s_val = unmark(sv) as usize;
        let gstart = isa[s_val] as usize;
        let mut num = (gend as usize) + 1 - gstart;

        if num == 1 {
            isa[s_val] = mark(gstart as u32);
            let p_raw = pss[s_val];
            let p = unmark(p_raw) as usize;
            if p < n {
                let gs = isa[p] as usize;
                if !is_marked(isa[p]) {
                    if sa[gs] == 1 && is_marked(p_raw) {
                        sa[gs] = mark(p as u32);
                    } else if sa[gs] != 1 {
                        sa[gs] = sa[gs].wrapping_sub(1);
                        let pos = (gs as u32).wrapping_add(sa[gs]) as usize;
                        sa[pos] = if is_marked(p_raw) { mark(p as u32) } else { 1 };
                        isa[p] = pos as u32;
                    }
                }
            }
            gend -= 1;
            continue;
        }

        gstarts.push(gstart as u32);
        let mut num_factors = 0;

        while num_factors < num {
            let item = unmark(sa[gstart + num_factors]) as usize;
            if unmark(pss[item]) == n as u32 { num_factors += 1; } else { break; }
        }

        if num_factors > 0 {
            for i in num_factors..num {
                let item = unmark(sa[gstart + i]) as usize;
                sa[gstart + i - num_factors] = pss[item] & (UMASK | MSB);
            }
            num -= num_factors;
            gend -= num_factors as isize;
            if num == 0 { gend = (gstart as isize) - 1; continue; }
        } else {
            for i in 0..num {
                let item = unmark(sa[gstart + i]) as usize;
                sa[gstart + i] = pss[item] & (UMASK | MSB);
            }
        }

        let mut elements = vec![0; num];
        for i in 0..num { elements[i] = sa[gstart + i]; }
        elements.sort_unstable_by_key(|&x| unmark(x));

        let mut singles_lc = Vec::new();
        let mut singles_nlc = Vec::new();
        let mut non_singles = Vec::new();
        let mut i = 0;

        while i < num {
            let val = elements[i];
            let p = unmark(val);
            let mut cnt = 0;
            let mut is_lc = false;
            while i + cnt < num && unmark(elements[i + cnt]) == p {
                if is_marked(elements[i + cnt]) { is_lc = true; }
                cnt += 1;
            }

            if cnt == 1 {
                if is_lc { singles_lc.push(elements[i]); } else { singles_nlc.push(elements[i]); }
            } else {
                let key = (cnt - 1) * 2 + (if is_lc { 0 } else { 1 });
                for k in 0..cnt { non_singles.push((elements[i + k], key)); }
            }
            i += cnt;
        }

        non_singles.sort_unstable_by_key(|x| x.1);

        let mut idx = 0;
        for &val in &singles_lc { sa[gstart + idx] = val; idx += 1; }
        for &val in &singles_nlc { sa[gstart + idx] = val; idx += 1; }
        for &(val, _) in &non_singles { sa[gstart + idx] = val; idx += 1; }

        let mut buckets = Vec::new();
        let mut cur = 0;
        if !singles_lc.is_empty() { buckets.push((0, 0, singles_lc.len())); }
        cur += singles_lc.len();
        if !singles_nlc.is_empty() { buckets.push((1, cur, cur + singles_nlc.len())); }
        cur += singles_nlc.len();

        let mut prev_key = -1;
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
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]) as usize;
                    let p_raw = isa[s];

                    if is_marked(p_raw) {
                        let pos = unmark(p_raw) as usize;
                        sa[pos] = mark(s as u32);
                        isa[s] = pos as u32;
                    } else {
                        let p = p_raw as usize;
                        sa[p] = sa[p].wrapping_sub(1);
                        let pos = (p as u32).wrapping_add(sa[p]) as usize;
                        sa[pos] = mark(s as u32);
                        isa[s] = if is_marked(sa[p]) { p as u32 } else { pos as u32 };
                    }
                }
            } else {
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]) as usize;
                    let p_raw = isa[s];
                    if !is_marked(p_raw) {
                        let p = p_raw as usize;
                        sa[p] = sa[p].wrapping_sub(1);
                    }
                }
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]) as usize;
                    let p_raw = isa[s];
                    let new_start = if is_marked(p_raw) {
                        unmark(p_raw) as usize
                    } else {
                        p_raw.wrapping_add(sa[p_raw as usize]) as usize
                    };
                    isa[s] = new_start as u32;
                    sa[new_start] = 0;
                }
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]) as usize;
                    let p_raw = isa[s] as usize;
                    sa[p_raw] += 1;
                }
            }
        }
        gend = (gstart as isize) - 1;
    }

    for (i, &gs) in gstarts.iter().enumerate() { sa[gs as usize] = i as u32; }
    for i in 0..n {
        let p_isa = isa[i];
        if !is_marked(p_isa) { isa[i] = sa[p_isa as usize]; }
    }

    println!("\r[INFO] Phase I Progress: 100%      ");
    gstarts
}