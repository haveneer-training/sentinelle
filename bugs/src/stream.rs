//! `stream` : comptage des échecs en flux. Un lecteur envoie des blocs de lignes, des
//! travailleurs les comptent, le thread principal agrège.

use crate::load::failed_or_accepted;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

const BLOCK: usize = 5_000;

pub fn run(dir: &Path, workers: usize) {
    let file = std::fs::File::open(dir.join("auth.log")).expect("auth.log");
    let (block_tx, block_rx) = mpsc::sync_channel::<Vec<String>>(workers * 2);
    let block_rx = Arc::new(Mutex::new(block_rx));
    let (count_tx, count_rx) = mpsc::channel::<HashMap<String, usize>>();

    let reader = thread::spawn(move || {
        let mut block = Vec::with_capacity(BLOCK);
        for line in BufReader::new(file).lines() {
            let Ok(line) = line else { continue };
            block.push(line);
            if block.len() == BLOCK {
                block_tx.send(std::mem::take(&mut block)).unwrap();
            }
        }
        if !block.is_empty() {
            block_tx.send(block).unwrap();
        }
    });

    for _ in 0..workers {
        let block_rx = Arc::clone(&block_rx);
        let count_tx = count_tx.clone();
        thread::spawn(move || {
            loop {
                let received = block_rx.lock().unwrap().recv();
                let Ok(block) = received else { break };
                let mut counts = HashMap::new();
                for line in &block {
                    if let Some((true, _, ip)) = failed_or_accepted(line) {
                        *counts.entry(ip.to_string()).or_insert(0) += 1;
                    }
                }
                count_tx.send(counts).unwrap();
            }
        });
    }

    let mut total: HashMap<String, usize> = HashMap::new();
    let mut blocks = 0;
    for counts in count_rx {
        blocks += 1;
        for (ip, n) in counts {
            *total.entry(ip).or_insert(0) += n;
        }
        eprint!("\r{blocks} blocs agrégés");
    }
    reader.join().unwrap();
    eprintln!();
    let mut top: Vec<_> = total.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (ip, count) in top.iter().take(5) {
        println!("{ip:>18} {count:>6} échecs");
    }
}
