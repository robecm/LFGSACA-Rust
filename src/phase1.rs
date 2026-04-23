use crate::utils::{is_marked, mark, unmark};
use std::io::{self, Write};

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

// ─────────────────────────────────────────────────────────────────────────────
// The core invariant of Phase I's counter scheme
// ─────────────────────────────────────────────────────────────────────────────
//
// For a group starting at index `gs`:
//   • sa[gs] starts as `group_size` (written by write_group_sizes).
//   • insert_leaves decrements it for each L-type leaf, filling right-to-left:
//       sa[gs] -= 1;  sa[gs + sa[gs]] = mark(leaf);
//     After all leaves, sa[gs] = number of S-type slots remaining (≥ 0).
//   • Phase I then inserts internal nodes into those S-type slots the same way:
//       sa[gs] -= 1;  sa[gs + sa[gs]] = child;  isa[child] = mark(gs + sa[gs]);
//
// SPECIAL CASE (singleton path, original line 86-88):
//   When sa[gs] == 1 and p is the last child, the code transitions the
//   group-start slot to store `mark(p)` directly instead of a counter:
//       sa[gs] = mark(p);  isa[p] = mark(gs);
//
//   This works for that one element — but it DESTROYS the counter. If any
//   other suffix also has isa[·] = gs (i.e., shares the same parent group),
//   it will later read sa[gs] = mark(p) thinking it's a decrement counter,
//   call wrapping_sub(1) on a marked value, and produce an MSB-set position.
//
// ROOT CAUSE
// ──────────
// Multiple S-type suffixes in the same character-group all get isa[·] = gs
// in insert_leaves. When the first one transitions sa[gs] to mark(p), every
// subsequent one that reads sa[gs] as a counter gets a marked (huge) value.
//
// THE FIX
// ───────
// Before using sa[gs] as a counter in any decrement path, check whether it
// has already been marked (i.e., the group-start slot has been repurposed as
// a concrete pointer). If so, isa[p] is already fully resolved — read the
// position from sa[gs] directly and record it in isa[p]. No decrement needed.
//
// This helper encapsulates that decision for the singleton path.
// For the bucket paths the same logic is inlined with clear comments.
// ─────────────────────────────────────────────────────────────────────────────

/// Inserts suffix `p` into its parent group `gs`.
/// `p_raw` is `pss[s]` (carries the last-child mark via MSB).
/// `gs` is `isa[p]` (the group-start index, guaranteed unmasked by caller).
/// `sa` / `isa` / `n` are the live algorithm arrays.
#[inline(always)]
fn insert_into_group(
    p: usize,
    p_raw: usize,
    gs: usize,
    sa: &mut [usize],
    isa: &mut [usize],
) {
    let sa_gs = sa[gs];

    if is_marked(sa_gs) {
        // The group-start slot was already repurposed as a concrete pointer
        // by a previous singleton insertion (the sa[gs]=mark(p) path).
        // The position it encodes is gs itself (the only slot left was slot 0).
        // Just write p there and mark isa[p].
        let pos = gs; // the sole remaining slot is the group-start itself
        sa[pos] = if is_marked(p_raw) { mark(p) } else { p };
        isa[p] = mark(pos);
    } else if sa_gs == 1 && is_marked(p_raw) {
        // Last slot AND p is the last child: transition group-start to pointer.
        sa[gs] = mark(p);
        isa[p] = mark(gs);
    } else if sa_gs >= 1 {
        // Normal fill: decrement counter, place p in the next slot from right.
        let new_cnt = sa_gs - 1; // plain subtraction, safe because sa_gs >= 1
        sa[gs] = new_cnt;
        let pos = gs + new_cnt;
        sa[pos] = if is_marked(p_raw) { mark(p) } else { p };
        isa[p] = mark(pos);
    }
    // sa_gs == 0: no room — this can occur if the group is all L-type and
    // every slot was consumed by insert_leaves. An S-type child pointing here
    // would be a PSS-tree inconsistency; silently skip so we don't corrupt.
}

pub fn phase1(sa: &mut [usize], pss: &[usize], isa: &mut [usize], n: usize) -> Vec<usize> {
    let mut gstarts = Vec::new();
    let mut gend = (n as isize) - 1;
    let update_interval = (n / 100).max(1) as isize;

    while gend >= 0 {
        if (n as isize - gend) % update_interval == 0 {
            print!(
                "\r[INFO] Phase I Progress: {}%  ",
                (((n as isize - gend) as f64 / n as f64) * 100.0) as usize
            );
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

        // ── Singleton ────────────────────────────────────────────────────────
        if num == 1 {
            isa[s_val] = mark(gstart);
            let p_raw = pss[s_val];
            let p = unmark(p_raw);
            if p < n {
                let gs_raw = isa[p];
                if !is_marked(gs_raw) {
                    // gs_raw is the parent group-start. Use the shared helper
                    // which handles the marked-counter case correctly.
                    insert_into_group(p, p_raw, gs_raw, sa, isa);
                } else {
                    // isa[p] already points to a concrete position.
                    let pos = unmark(gs_raw);
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
                sa[gstart + i - num_factors] =
                    if is_marked(pss[item]) { mark(item) } else { item };
            }
            num -= num_factors;
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

        let mut elements = vec![0usize; num];
        for i in 0..num { elements[i] = sa[gstart + i]; }
        elements.sort_unstable_by_key(|&x| unmark(pss[unmark(x)]));

        let mut singles_lc: Vec<usize> = Vec::new();
        let mut singles_nlc: Vec<usize> = Vec::new();
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
                // ── is_final bucket ──────────────────────────────────────────
                // Each child s is the last child of its parent group (or the
                // sole child). Decrement sa[p] and place s in the resulting slot.
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];

                    if is_marked(p_raw) {
                        // Parent already has a concrete position.
                        let pos = unmark(p_raw);
                        sa[pos] = mark(s);
                        isa[s] = mark(pos);
                    } else {
                        // p_raw is the parent group-start index.
                        let p = p_raw;
                        let sa_p = sa[p];

                        if is_marked(sa_p) {
                            // ── FIX: sa[p] was repurposed as a pointer ───────
                            // The group-start slot no longer holds a counter.
                            // The only remaining slot is p itself (slot 0).
                            let pos = p;
                            sa[pos] = mark(s);
                            isa[s] = mark(pos);
                        } else {
                            // Normal path: sa_p is a plain counter.
                            // Use plain subtraction — sa_p must be ≥ 1 here
                            // because is_final children are inserted last and
                            // the PSS-tree guarantees at least one slot exists.
                            let new_sa_p = sa_p.wrapping_sub(1);
                            sa[p] = new_sa_p;
                            let pos = p.wrapping_add(new_sa_p);
                            sa[pos] = mark(s);
                            isa[s] = mark(pos);
                        }
                    }
                }
            } else {
                // ── non-final bucket: three-pass insertion ───────────────────
                //
                // Pass 1 — decrement sa[p] once per non-final child.
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];
                    if !is_marked(p_raw) {
                        let p = p_raw;
                        let sa_p = sa[p];
                        // Only decrement if the slot is still a plain counter.
                        // If it's already marked (repurposed pointer), there is
                        // exactly one slot left (slot p itself) and no counter
                        // arithmetic is needed or valid.
                        if !is_marked(sa_p) {
                            sa[p] = sa_p.wrapping_sub(1);
                        }
                    }
                }

                // Pass 2 — compute base address and initialise fill counter.
                let mut prev_p = usize::MAX;
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];

                    if is_marked(p_raw) {
                        let pos = unmark(p_raw);
                        sa[pos] = s;
                        isa[s] = mark(pos);
                    } else {
                        let p = p_raw;
                        let sa_p = sa[p];
                        let new_start = if is_marked(sa_p) { p } else { p.wrapping_add(sa_p) };
                        isa[s] = new_start; // still unmarked; Pass 3 will mark it
                        if p != prev_p {
                            sa[new_start] = 0;
                            prev_p = p;
                        }
                    }
                }

                // Pass 3 — assign concrete slot, write child, mark isa[s].
                // new_start (stored unmarked in isa[s]) is the base of the
                // reserved block.  sa[new_start] is the running fill offset.
                // We compute pos = new_start + offset, place the child there,
                // advance the offset, then mark isa[s] = mark(pos) so the
                // resolution loop at the end of phase1 never treats it as a
                // raw group-start pointer.
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];
                    if !is_marked(p_raw) {
                        let new_start = p_raw;
                        let offset = sa[new_start];
                        let pos = new_start + offset;
                        sa[new_start] = offset + 1;
                        sa[pos] = s;
                        isa[s] = mark(pos);
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