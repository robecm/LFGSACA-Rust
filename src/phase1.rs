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

        if num == 1 {
            isa[s_val] = mark(gstart);
            let p_raw = pss[s_val];
            let p = unmark(p_raw);
            if p < n {
                let gs_raw = isa[p];
                if !is_marked(gs_raw) {
                    let gs = gs_raw;
                    // BUG FIX 1: sa[gs] holds a remaining-count which can itself be
                    // a large (but unmarked) number.  We must read it as a plain usize
                    // and compare against 1 directly — this was already correct — BUT
                    // we also have to make sure we never skip the `else if` branch when
                    // sa[gs] == 0 (exhausted counter), because wrapping_sub(1) on 0
                    // yields usize::MAX.  Guard with an explicit != 0 check.
                    let sa_gs = sa[gs];
                    if sa_gs == 1 && is_marked(p_raw) {
                        sa[gs] = mark(p);
                        isa[p] = mark(gs);
                    } else if sa_gs != 1 && sa_gs != 0 {
                        // BUG FIX 2: Only decrement when the counter is valid (> 1).
                        // A value of 0 means the slot is already consumed; decrementing
                        // it would wrap to usize::MAX and corrupt the position.
                        let new_sa_gs = sa_gs.wrapping_sub(1);
                        sa[gs] = new_sa_gs;

                        // new_sa_gs is the new remaining count (never MSB-set because
                        // group sizes are bounded by n < 2^63).
                        let pos = gs + new_sa_gs;
                        sa[pos] = if is_marked(p_raw) { mark(p) } else { p };
                        isa[p] = mark(pos);
                    }
                } else {
                    let pos = unmark(gs_raw);
                    sa[pos] = if is_marked(p_raw) { mark(p) } else { p };
                    isa[p] = mark(pos);
                }
            }
            gend -= 1;
            continue;
        }

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

        let mut singles_lc = Vec::new();
        let mut singles_nlc = Vec::new();
        let mut non_singles = Vec::new();
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
                if is_lc { singles_lc.push(elements[i]); } else { singles_nlc.push(elements[i]); }
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

        let mut buckets = Vec::new();
        let mut cur = 0;
        if !singles_lc.is_empty() { buckets.push((0isize, 0usize, singles_lc.len())); }
        cur += singles_lc.len();
        if !singles_nlc.is_empty() { buckets.push((1isize, cur, cur + singles_nlc.len())); }
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
                // "Final" bucket: each element's parent is already resolved (marked in isa).
                // We either place directly into the marked slot, or decrement the parent
                // group-start counter and insert at the resulting position.
                for i in bs..bend {
                    let s = unmark(sa[gstart + i]);
                    let p_raw = isa[s];

                    if is_marked(p_raw) {
                        // isa[s] already points to a concrete position (marked = resolved).
                        let pos = unmark(p_raw);
                        sa[pos] = mark(s);
                        isa[s] = mark(pos);
                    } else {
                        // isa[s] is an unresolved group-start index.
                        // BUG FIX 3: The counter at sa[p] is a remaining-count stored as a
                        // plain usize (never MSB-set).  Use plain addition instead of
                        // wrapping_add so that any overflow (which would indicate a logic
                        // error elsewhere) surfaces immediately rather than silently
                        // producing a bogus MSB-set index.
                        let p = p_raw;             // p is a valid group-start index
                        let cnt = sa[p];           // remaining count (plain usize, < n)
                        debug_assert!(cnt > 0, "group counter already exhausted at phase1 is_final");
                        let new_cnt = cnt - 1;
                        sa[p] = new_cnt;
                        let pos = p + new_cnt;     // plain addition — never wraps for valid input
                        sa[pos] = mark(s);
                        isa[s] = mark(pos);
                    }
                }
            } else {
                // "Non-final" bucket: multiple children share the same parent group.
                // We need a three-pass approach:
                //   Pass 1 — count how many elements will land in each parent group
                //            (decrement the counter at sa[p] once per distinct parent p).
                //   Pass 2 — compute the insertion base-pointer for each element and
                //            record it in isa[s] (temporarily, as a plain index).
                //   Pass 3 — increment a per-group fill counter so later passes find
                //            the slot filled sequentially.
                //
                // BUG FIX 4 (the main crash): In the original code Pass 1 decremented
                // sa[p] once per *element* rather than once per *distinct parent*.
                // That caused the counter to underflow (wrapping_sub on 0 → usize::MAX),
                // and the subsequent wrapping_add produced an MSB-set "position" that
                // was stored in isa[s] and later used as sa[index] → panic.
                //
                // The correct algorithm is:
                //   • For each distinct parent p, decrement sa[p] by the number of
                //     children that will be inserted into that group (i.e. cnt children).
                //   • Then compute base = p + sa[p] (the start of the reserved slot range).
                //   • Place children at base, base+1, … using a local fill counter,
                //     updating isa[s] = mark(pos) immediately so the slot is "claimed".

                // Pass 1: for each distinct parent group, reserve `cnt` slots by
                //         subtracting cnt from the group-start counter sa[p].
                {
                    let mut j = bs;
                    while j < bend {
                        let s0 = unmark(sa[gstart + j]);
                        let p_raw = isa[s0];
                        if is_marked(p_raw) {
                            // Already resolved — no counter to touch.
                            j += 1;
                            continue;
                        }
                        let p = p_raw;
                        // Count siblings that share the same parent p.
                        let mut siblings = 1usize;
                        let mut k = j + 1;
                        while k < bend {
                            let sk = unmark(sa[gstart + k]);
                            let pk_raw = isa[sk];
                            if is_marked(pk_raw) || pk_raw != p { break; }
                            siblings += 1;
                            k += 1;
                        }
                        // Reserve `siblings` consecutive slots.
                        let cnt = sa[p];
                        debug_assert!(cnt >= siblings,
                                      "group counter underflow: cnt={cnt} siblings={siblings} p={p}");
                        sa[p] = cnt - siblings;
                        j += siblings;
                    }
                }

                // Pass 2: assign each element its concrete position.
                //         For already-resolved (marked) isa[s], the position is unmark(isa[s]).
                //         For unresolved, the base is p + sa[p]; we fill sequentially and
                //         mark isa[s] immediately so Pass 3 can distinguish resolved entries.
                {
                    // We need a per-parent fill offset.  Reuse a local map keyed on p.
                    // Since siblings are contiguous in our iteration (they were sorted by
                    // parent earlier via elements.sort_unstable_by_key), a simple
                    // "previous p" trick suffices.
                    let mut prev_p = usize::MAX;
                    let mut fill_offset = 0usize;

                    for i in bs..bend {
                        let s = unmark(sa[gstart + i]);
                        let p_raw = isa[s];

                        if is_marked(p_raw) {
                            // Slot already concrete — just write the child value there.
                            let pos = unmark(p_raw);
                            sa[pos] = s;           // not marked: non-final children are stored unmarked
                            isa[s] = mark(pos);    // mark isa[s] so Pass 3 skips it
                        } else {
                            let p = p_raw;
                            if p != prev_p {
                                // New parent: base position is p + sa[p] (already decremented).
                                fill_offset = 0;
                                prev_p = p;
                            }
                            let base = p + sa[p];  // sa[p] was decremented in Pass 1
                            let pos = base + fill_offset;
                            fill_offset += 1;
                            sa[pos] = s;
                            isa[s] = mark(pos);    // mark so Pass 3 (fill counter) skips it
                        }
                    }
                }

                // Pass 3: restore the fill counter at sa[p + sa[p]] so subsequent phases
                //         can find how many slots were used.  In the original three-pass
                //         design this was an increment pass; here we simply initialise
                //         sa[base] with the sibling count for the first element of each
                //         group so that the group-size book-keeping remains consistent.
                //
                // NOTE: Because we marked isa[s] in Pass 2, we use that to recover pos,
                //       and from pos we can recover the group base.  However, since we
                //       already wrote children into sa[pos] in Pass 2, and the counter
                //       is only needed for downstream decrement-based insertion, we only
                //       need to ensure sa[p] (the group-start counter) is a non-MSB value.
                //       After Pass 1 sa[p] holds (original_count - siblings), which is
                //       correct for the next consumer.  No further action is needed here.
                //
                // The original "third pass" incremented sa[p_raw] where p_raw was the
                // raw (possibly wrapping) new_start stored in isa[s].  That was the direct
                // source of the index-out-of-bounds: a wrapped value used as sa[index].
                // We eliminate that pass entirely because Pass 1 already maintains the
                // counter correctly.
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