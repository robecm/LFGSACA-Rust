use std::fs;
use std::path::Path;
use std::time::Instant;

// Importamos la función desde la raíz de nuestro paquete (librería)
use LFGSACA::fgsaca;

fn main() {
    println!("--- FGSACA Algorithm: Modular Rust Implementation Benchmark ---");

    let folder_path = "D:/1. PC-Real";

    if Path::new(folder_path).exists() {
        println!("[INFO] Starting benchmark sequence in: {}", folder_path);

        if let Ok(entries) = fs::read_dir(folder_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_size = fs::metadata(&path).unwrap().len();

                    if file_size > 300_000_000 {
                        println!(
                            "\n[WARN] Skipping '{}' ({} bytes) - File exceeds 300MB memory safety limit.",
                            path.file_name().unwrap().to_string_lossy(),
                            file_size
                        );
                        continue;
                    }

                    println!("\n[INFO] Processing: {}", path.display());
                    if let Ok(content) = fs::read(&path) {
                        let n = content.len();
                        println!("[INFO] Input size: {} bytes", n);

                        let start = Instant::now();
                        let _sa = fgsaca(&content);
                        let duration = start.elapsed();

                        println!("[SUCCESS] Execution time: {:.4} seconds", duration.as_secs_f64());
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