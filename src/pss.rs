use crate::utils::mark;
use std::io::{self, Write};

pub fn compute_ls_types(s: &[u8], n: usize) -> Vec<u8> {
    let mut types = vec![0; n];
    let mut t = 0;
    let mut c1 = s[n - 1];

    for i in (0..n - 1).rev() {
        let c0 = s[i];
        t = if (c0 as u16) < (c1 as u16) + t { 1 } else { 0 };
        types[i] = t as u8;
        c1 = c0;
    }
    types
}

pub fn compute_pss(text: &[u8], n: usize) -> Vec<usize> {
    let mut pss = vec![n as usize; n];
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let update_interval = (n / 10).max(1);

    for i in 0..n {
        if i % update_interval == 0 {
            print!(
                "\r[INFO] PSS Computation: {}%  ",
                ((i as f64 / n as f64) * 100.0) as usize
            );
            io::stdout().flush().unwrap();
        }

        while let Some(&j_val) = stack.last() {
            let j = j_val as usize;
            let li = n - i;
            let lj = n - j;
            let mn = li.min(lj);

            let mut lcp = 0;
            while lcp < mn {
                if text[j + lcp] != text[i + lcp] {
                    break;
                }
                lcp += 1;
            }
            let less = if lcp < mn {
                text[j + lcp] < text[i + lcp]
            } else {
                lj < li
            };

            if less {
                break;
            }
            stack.pop();
        }
        pss[i] = if let Some(&j) = stack.last() {
            j
        } else {
            n as usize
        };
        stack.push(i as usize);
    }

    let mut last_child = vec![n as usize; n];
    for i in 0..n {
        let p = pss[i];
        if p != n as usize {
            last_child[p as usize] = i as usize;
        }
    }
    for parent in 0..n {
        let child = last_child[parent];
        if child != n as usize {
            pss[child as usize] = mark(pss[child as usize]);
        }
    }

    println!("\r[INFO] PSS Computation: 100%      ");
    pss
}
