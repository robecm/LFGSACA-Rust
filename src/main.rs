use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;
use lfgsaca_rust::{fgsaca, FgsacaMode};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("[ERROR] Uso: ./lfgsaca <ruta_archivo> [--bbwt | --ebwt]");
        return;
    }

    let mode = if args.contains(&"--bbwt".to_string()) {
        FgsacaMode::BBWT
    } else if args.contains(&"--ebwt".to_string()) {
        FgsacaMode::EBWT
    } else {
        FgsacaMode::SuffixArray
    };

    let file_path = &args[1];
    let path = Path::new(file_path);

    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len();
        let file = File::open(path).expect("[ERROR] Error abriendo archivo");
        let mut reader = BufReader::new(file);
        let mut content = Vec::with_capacity(size as usize);
        reader.read_to_end(&mut content).expect("[ERROR] Error leyendo archivo");

        let start = Instant::now();
        let _sa = fgsaca(&content, mode);
        let duration = start.elapsed();

        println!("[INFO] Size: {} bytes", size);
        println!("[SUCCESS] Time: {:.4} s", duration.as_secs_f64());
    } else {
        println!("[ERROR] Archivo no encontrado: {}", file_path);
    }
}