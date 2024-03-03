use flate2::read::GzDecoder;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
/// Represents a collection of file paths and corresponding headers.
pub struct Samples {
    paths: Vec<String>,
    headers: Vec<String>,
}

/// nodes counts result
pub struct Nodes {
    nod: usize,
    count: usize,
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
                "Invalid node value",
            )
        })?;
        let count = parts[1].parse::<usize>().map_err(|_| {
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
    ) -> io::Result<Arc<Mutex<Vec<(String, String, HashSet<usize>)>>>> {
        let all_nodes: Arc<Mutex<Vec<(String, String, HashSet<usize>)>>> =
            Arc::new(Mutex::new(Vec::new()));

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
                    if path_obj.extension().and_then(|s| s.to_str())
                        == Some("gz")
                    {
                        Box::new(BufReader::new(GzDecoder::new(file)))
                    } else {
                        Box::new(BufReader::new(file))
                    };

                // Iterate over the lines of the file, parsing each and collecting relevant nodes.
                let mut nodes = HashSet::new();
                for line_result in reader.lines() {
                    let line = line_result?;
                    let node_data = Nodes::from_line(&line)?;
                    if node_data.count >= 2 {
                        nodes.insert(node_data.nod);
                    }
                }

                // Safely access the shared structure to store the results.
                let mut all_nodes_lock = all_nodes.lock().map_err(|_| {
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
            })?;

        Ok(all_nodes)
    }

    pub fn merge_write(
        &self,
        all_nodes: io::Result<
            Arc<Mutex<Vec<(String, String, HashSet<usize>)>>>,
        >,
        output_file_path: &str,
    ) -> io::Result<()> {
        let all_nodes = all_nodes?;
        let all_nodes_lock = all_nodes.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Mutex is poisoned")
        })?;
        let headers = all_nodes_lock
            .iter()
            .map(|i| i.1.clone())
            .collect::<Vec<String>>();

        let output_file = File::create(output_file_path)?;
        let mut writer = BufWriter::new(output_file);

        // Write header
        writeln!(writer, "node\t{}", headers.join("\t"))?;

        let all_unique_nodes: Vec<usize> = all_nodes_lock
            .iter()
            .flat_map(|(_, _, nodes_set)| nodes_set)
            .cloned()
            .collect();
        // all_unique_nodes.sort_unstable();

        // Iterate over each unique node and write line by line
        for node in all_unique_nodes {
            let mut line = vec![node.to_string()];
            for (_, _, nodes_set) in &*all_nodes_lock {
                line.push(
                    if nodes_set.contains(&node) { "1" } else { "0" }
                        .to_string(),
                );
            }
            writeln!(writer, "{}", line.join("\t"))?;
        }

        writer.flush()?;
        Ok(())
    }
}
