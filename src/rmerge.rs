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
    pub fn from_line(line: &str) -> Result<Self, std::io::Error> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Line does not contain exactly two elements",
            ));
        }

        let nod = parts[0].parse::<usize>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid node ID",
            )
        })?;
        let count = parts[1].parse::<u32>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid count value",
            )
        })?;

        Ok(Nodes { nod, count })
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

    pub fn max_value(&self) -> Option<usize> {
        let vals: Vec<Option<usize>> = self
            .paths
            .par_iter()
            .map(|path| {
                // if file not exist, return None
                let file = File::open(path).ok()?;
                let reader: Box<dyn BufRead> =
                    if gzip_true(path).expect("Failed to check gzip") {
                        Box::new(BufReader::new(GzDecoder::new(file)))
                    } else {
                        Box::new(BufReader::new(file))
                    };
                let mut local_max = 0;

                for line in reader.lines() {
                    let line = line.ok()?;
                    let parts: Vec<&str> =
                        line.trim().split_whitespace().collect();
                    if let Some(count) =
                        parts.get(0).and_then(|c| c.parse::<usize>().ok())
                    {
                        if count > local_max {
                            local_max = count;
                            // println!("{}", local_max);
                        }
                    }
                }
                Some(local_max) // return None if file not exist
            })
            .collect();
        let mut max_val = 0_usize;
        for val in vals {
            if let Some(v) = val {
                if v > max_val {
                    max_val = v;
                }
            }
        }
        Some(max_val)
    }

    /// Constructs a `Samples` instance from a given file, parsing paths and headers.
    pub fn from_paths<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut samples = Samples::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() != 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Line does not contain exactly two elements",
                ));
            }
            samples.paths.push(parts[0].to_string());
            samples.headers.push(parts[1].to_string());
        }

        Ok(samples)
    }
    pub fn path_to_sets(
        &self,
        max_val: usize,
    ) -> io::Result<Arc<Mutex<Vec<(String, String, Vec<u32>)>>>> {
        let all_nodes: Arc<Mutex<Vec<(String, String, Vec<u32>)>>> =
            Arc::new(Mutex::new(Vec::with_capacity(self.paths.len())));
        let pool = ThreadPoolBuilder::new().num_threads(10).build().unwrap();
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
                    for line_result in reader.lines() {
                        let line = line_result?;
                        let node_data = Nodes::from_line(&line)?;
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
        num: u32,
        transpose: bool,
    ) -> io::Result<()> {
        let all_nodes = all_nodes?;
        let all_nodes_lock = all_nodes.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Mutex is poisoned")
        })?;
        let headers = all_nodes_lock
            .iter()
            .map(|i| i.1.clone())
            .collect::<Vec<String>>();

        let output_file = File::create(format!("{}.gz", output_file_path))?;
        let encoder = GzEncoder::new(output_file, Compression::default());
        let mut writer = BufWriter::new(encoder);

        // Write header
        writeln!(writer, "node\t{}", headers.join("\t"))?;

        let max_val = all_nodes_lock[0].2.len();

        // if transpose, write node_id and presence ,1 equal to represent and 0 equal to absent
        // if not transpose, write node_id and count
        if transpose {
            for index in 0..max_val {
                let mut line = (index + 1).to_string();
                for (_, _, nodes_set) in &*all_nodes_lock {
                    line.push('\t');
                    // The count of node is greater than or equal to num, then write 1, otherwise write 0.
                    let presence =
                        if nodes_set[index] >= num { '1' } else { '0' };
                    line.push(presence);
                }
                writeln!(writer, "{}", line)?;
            }
        } else {
            for index in 0..max_val {
                let mut line = (index + 1).to_string();
                for (_, _, nodes_set) in &*all_nodes_lock {
                    line.push('\t');
                    let tem = nodes_set[index].to_string();
                    line += &tem;
                }
                writeln!(writer, "{}", line)?;
            }
        }

        writer.flush()?;
        Ok(())
    }
}
