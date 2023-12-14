use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

pub fn filter_ids(input_file: &str, output_file: &str) {
    // record
    let mut id_counts: HashMap<String, usize> = HashMap::new();

    if let Ok(file) = File::open(input_file) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(id) = line {
                *id_counts.entry(id).or_insert(0) += 1;
            }
        }
    } else {
        eprintln!("Can't open {}", input_file);
        std::process::exit(1);
    }

    // retain  2 >=
    if let Ok(file) = File::create(output_file) {
        let mut writer = BufWriter::new(file);
        for (id, count) in &id_counts {
            if *count >= 2 {
                writeln!(writer, "{}", id).expect("write result error !");
            }
        }
    } else {
        eprintln!("Can't create {}", output_file);
        std::process::exit(1);
    }
}
