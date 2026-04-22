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
    let mut pss = vec![n; n];
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let update_interval = (n / 100).max(1);

    for i in 0..n {
        if i % update_interval == 0 {
            print!("\r[INFO] PSS Computation: {}%  ", ((i as f64 / n as f64) * 100.0) as usize);
            io::stdout().flush().unwrap();
        }

        while let Some(&j) = stack.last() {
            if compare_suffixes(text, j, i, n) { break; }
            stack.pop();
        }
        pss[i] = stack.last().cloned().unwrap_or(n);
        stack.push(i);
    }

    println!("\r[INFO] PSS Computation: 100%      ");
    mark_last_children(&mut pss, n);
    pss
}

pub fn compute_lpss(text: &[u8], n: usize) -> Vec<usize> {
    let mut pss = vec![n; n];
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let update_interval = (n / 100).max(1);

    for i in 0..n {
        if i % update_interval == 0 {
            print!("\r[INFO] LPSS Computation: {}%  ", ((i as f64 / n as f64) * 100.0) as usize);
            io::stdout().flush().unwrap();
        }

        if text[i] == b'$' { stack.clear(); }

        while let Some(&j) = stack.last() {
            if compare_suffixes(text, j, i, n) { break; }
            stack.pop();
        }
        pss[i] = stack.last().cloned().unwrap_or(n);
        stack.push(i);
    }

    println!("\r[INFO] LPSS Computation: 100%      ");
    mark_last_children(&mut pss, n);
    pss
}

fn compare_suffixes(text: &[u8], j: usize, i: usize, n: usize) -> bool {
    let len_j = n - j;
    let len_i = n - i;
    let common = len_j.min(len_i);
    for k in 0..common {
        if text[j + k] != text[i + k] {
            return text[j + k] < text[i + k];
        }
    }
    len_j < len_i
}

fn mark_last_children(pss: &mut [usize], n: usize) {
    let mut last_child = vec![n; n];
    for i in 0..n {
        let p = pss[i];
        if p != n { last_child[p] = i; }
    }
    for p in 0..n {
        let c = last_child[p];
        if c != n { pss[c] = mark(pss[c]); }
    }
}