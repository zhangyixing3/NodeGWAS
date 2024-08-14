use bstr::io::BufReadExt;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use rayon::ThreadPoolBuilder;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

const THREAD_NUM: usize = 5;
fn gzip_true(filepath: &str) -> io::Result<bool> {
    let mut infile = File::open(filepath)?;
    let mut buf = [0u8; 2];
    infile.read_exact(&mut buf)?;
    Ok(buf[0] == 31 && buf[1] == 139)
}

/// Represents a collection of file paths and corresponding headers.
pub struct Samples {
    paths: Vec<String>,
    headers: Vec<String>,
}

/// nodes counts result
pub struct Nodes {
    nod: usize,
    count: u32,
}
impl Nodes {
    pub fn from_line(line: &Vec<u8>) -> Nodes {
        let left = line.iter().position(|&b| b == b'\t').unwrap();
        let node = &line[..left];
        let count = &line[left + 1..];
        let mut node_value = 0;
        for &i in node.iter() {
            node_value = node_value * 10 + (i - b'0') as usize;
        }
        let mut count_value = 0;
        for &i in count.iter() {
            count_value = count_value * 10 + (i - b'0') as u32;
        }
        Nodes {
            nod: node_value,
            count: count_value,
        }
    }
}

impl Samples {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            headers: Vec::new(),
        }
    }

    /// Validates the existence of each file path stored in the `paths` vector.
    /// Returns an error if any file does not exist.
    pub fn validate_paths(&self) -> io::Result<()> {
        for path in &self.paths {
            if !Path::new(path).is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{} does not exist", path),
                ));
            }
        }
        Ok(())
    }
    pub fn max_value(&self) -> usize {
        let vals: Vec<Vec<u8>> = self
            .paths
            .par_iter()
            .map(|path| {
                // if file not exist, return None
                let file = File::open(path).ok().unwrap();
                let reader: Box<dyn BufRead> =
                    if gzip_true(path).expect("Failed to check gzip") {
                        Box::new(BufReader::new(GzDecoder::new(file)))
                    } else {
                        Box::new(BufReader::new(file))
                    };
                let mut max_sample: Vec<u8> = vec![];
                for line in reader.byte_lines() {
                    let line = line.ok().unwrap();
                    let left = line.iter().position(|&b| b == b'\t').unwrap();
                    let b = &line[..left];
                    if b.len() > max_sample.len() || b > &max_sample[..] {
                        max_sample = b.to_vec();
                    }
                    // if a.len() < b.len() {
                    //     a = b.to_vec();
                    // } else if a.len() == b.len() {
                    //     for (byte_a, byte_b) in a.iter().zip(b.iter()) {
                    //         if byte_a < byte_b {
                    //             a = b.to_vec();
                    //             break;
                    //         }
                    //     }
                    // }
                }
                max_sample
            })
            .collect();
        // let mut a: Vec<u8> = vec![49];
        // for b in vals {
        //     if a.len() < b.len() {
        //         a = b.to_vec();
        //     } else if a.len() == b.len() {
        //         for (byte_a, byte_b) in a.iter().zip(b.iter()) {
        //             if byte_a < byte_b {
        //                 a = b.to_vec();
        //                 break;
        //             }
        //         }
        //     }
        // }
        let mut max_value: Vec<u8> = vec![];
        for b in vals {
            if b.len() > max_value.len() || &b[..] > &max_value[..] {
                max_value = b;
            }
        }
        let mut result = 0_usize;
        for &i in max_value.iter() {
            result = result * 10 + (i - b'0') as usize;
        }
        result
    }

    /// Constructs a `Samples` instance from a given file, parsing paths and headers.
    pub fn from_paths<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut samples = Samples::new();

        for line in reader.byte_lines() {
            let line = line?;
            let pos = line.iter().position(|&b| b == b'\t');
            match pos {
                Some(pos) => {
                    let path = String::from_utf8(line[..pos].to_vec()).unwrap();
                    let header =
                        String::from_utf8(line[pos + 1..].to_vec()).unwrap();
                    samples.paths.push(path);
                    samples.headers.push(header);
                }
                None => {
                    panic!("please check your input file, Can't find tab!");
                }
            }
        }
        Ok(samples)
    }
    pub fn path_to_sets(
        &self,
        max_val: usize,
    ) -> io::Result<Arc<Mutex<Vec<(String, String, Vec<u32>)>>>> {
        let all_nodes: Arc<Mutex<Vec<(String, String, Vec<u32>)>>> =
            Arc::new(Mutex::new(Vec::with_capacity(self.paths.len())));
        let pool = ThreadPoolBuilder::new()
            .num_threads(THREAD_NUM)
            .build()
            .unwrap();
        let result = pool.install(|| {
            self.paths
                .par_iter()
                .enumerate()
                .try_for_each(|(index, path)| {
                    // Obtain the header corresponding to the current file.
                    let header = &self.headers[index];

                    // Create a Path object and attempt to open the corresponding file.
                    let path_obj = Path::new(path);
                    let file = match File::open(path_obj) {
                        Ok(f) => f,
                        Err(e) => return Err(e),
                    };

                    // Determine if the file is gzipped and create the appropriate reader.
                    let reader: Box<dyn BufRead> =
                        if gzip_true(path).expect("Failed to check gzip") {
                            Box::new(BufReader::new(GzDecoder::new(file)))
                        } else {
                            Box::new(BufReader::new(file))
                        };

                    // Iterate over the lines of the file, parsing each and collecting relevant nodes.
                    let mut nodes = vec![0; max_val];
                    for line_result in reader.byte_lines() {
                        let line = line_result?;
                        let node_data = Nodes::from_line(&line);
                        if node_data.count >= 2 {
                            // nodes.insert(node_data.nod);
                            // println!("{} {}", node_data.nod, node_data.count);
                            nodes[node_data.nod - 1] = node_data.count;
                        }
                    }

                    // Safely access the shared structure to store the results.
                    let mut all_nodes_lock =
                        all_nodes.lock().map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Mutex lock error",
                            )
                        })?;
                    all_nodes_lock.push((
                        path.to_string(),
                        header.to_string(),
                        nodes,
                    ));

                    Ok(())
                })
        });
        result?;
        println!("finish merge...");
        Ok(all_nodes)
    }

    pub fn merge_write(
        &self,
        all_nodes: io::Result<Arc<Mutex<Vec<(String, String, Vec<u32>)>>>>,
        output_file_path: &str,
        value: u32,
        transpose: bool,
    ) -> io::Result<()> {
        let all_nodes = all_nodes?;
        let all_nodes_lock = all_nodes.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Mutex is poisoned")
        })?;
        // let headers = all_nodes_lock
        //     .iter()
        //     .map(|i| i.1.clone())
        //     .collect::<Vec<String>>();

        let output_file = File::create(format!("{}.gz", output_file_path))?;
        let encoder = GzEncoder::new(output_file, Compression::default());
        let mut writer = BufWriter::new(encoder);
        writer.write(b"node").unwrap();
        for header in all_nodes_lock.iter().map(|i| i.1.clone()) {
            writer.write(b"\t").unwrap();
            writer.write(header.as_bytes()).unwrap();
        }
        writer.write(b"\n").unwrap();
        // // Write header
        // writeln!(writer, "node\t{}", headers.join("\t"))?;

        let max_val = all_nodes_lock[0].2.len();

        // if transpose, write node_id and presence ,1 equal to represent and 0 equal to absent
        // if not transpose, write node_id and count
        if transpose {
            for index in 0..max_val {
                let digits = format!("{}", index + 1).into_bytes();
                writer.write(&digits).unwrap();

                for (_, _, nodes_set) in &*all_nodes_lock {
                    writer.write(b"\t").unwrap();
                    // The count of node is greater than or equal to num, then write 1, otherwise write 0.
                    let presence = if nodes_set[index] >= value {
                        b"1"
                    } else {
                        b"0"
                    };
                    writer.write(presence).unwrap();
                }
                writer.write(b"\n").unwrap();
            }
        } else {
            for index in 0..max_val {
                let digits = format!("{}", index + 1).into_bytes();
                writer.write(&digits).unwrap();
                for (_, _, nodes_set) in &*all_nodes_lock {
                    writer.write(b"\t").unwrap();
                    let byte_array =
                        format!("{}", nodes_set[index]).into_bytes();
                    writer.write(&byte_array).unwrap();
                }
                writer.write(b"\n").unwrap();
            }
        }
        writer.flush()?;
        Ok(())
    }
}
