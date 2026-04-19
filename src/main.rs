use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;
use LFGSACA::fgsaca;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Si el script de Bash nos pasa una ruta por argumento
    if args.len() > 1 {
        let file_path = &args[1];
        process_single_file(file_path);
    } else {
        println!("[ERROR] Uso: ./programa <ruta_archivo>");
        println!("[INFO] Ejecutando test rápido...");
        let sa = fgsaca(b"abracadabra");
        println!("[SUCCESS] Test output: {:?}", sa);
    }
}

fn process_single_file(file_path: &str) {
    let path = Path::new(file_path);
    if let Ok(metadata) = fs::metadata(path) {
        let file_size = metadata.len();
        println!("[INFO] Procesando: {} ({} bytes)", path.display(), file_size);

        if let Ok(file) = File::open(path) {
            let mut reader = BufReader::new(file);
            let mut content = Vec::with_capacity(file_size as usize);
            if let Ok(_) = reader.read_to_end(&mut content) {
                let start = Instant::now();
                let _sa = fgsaca(&content);
                let duration = start.elapsed();
                println!("[SUCCESS] Tiempo: {:.4} segundos", duration.as_secs_f64());
            }
        }
    } else {
        println!("[ERROR] No se pudo acceder al archivo: {}", file_path);
    }
}