use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;

// Importamos la función desde la raíz de nuestro paquete (librería)
use LFGSACA::fgsaca;

fn main() {
    println!("--- FGSACA Algorithm: Modular Rust Implementation Benchmark ---");

    let folder_path = "/home/roberto_carranzam08/datos_pia";;

    if Path::new(folder_path).exists() {
        println!("[INFO] Starting benchmark sequence in: {}", folder_path);

        if let Ok(entries) = fs::read_dir(folder_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_size = fs::metadata(&path).unwrap().len();

                    println!("\n[INFO] Processing: {}", path.display());
                    if let Ok(file) = File::open(&path) {
                        let mut reader = BufReader::new(file);
                        let mut content = Vec::with_capacity(file_size as usize);
                        if let Ok(_) = reader.read_to_end(&mut content) {
                            let n = content.len();
                            println!("[INFO] Input size: {} bytes", n);

                            let start = Instant::now();
                            let _sa = fgsaca(&content);
                            let duration = start.elapsed();

                            println!(
                                "[SUCCESS] Execution time: {:.4} seconds",
                                duration.as_secs_f64()
                            );
                        } else {
                            println!("[ERROR] Failed to read file {}", path.display());
                        }
                    } else {
                        println!("[ERROR] Failed to open file {}", path.display());
                    }
                }
            }
        }
    } else {
        println!("[ERROR] Directory not found. Executing verification test...");
        let text = b"abracadabra";
        let sa = fgsaca(text);
        println!("[INFO] Target: abracadabra");
        println!("[SUCCESS] Suffix Array Output: {:?}", sa);
    }
}
