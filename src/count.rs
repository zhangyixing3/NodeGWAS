use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;

#[derive(Deserialize)]
struct MyStruct {
    path: Option<Path>,
}
#[derive(Deserialize)]
struct Path {
    mapping: Option<Vec<Mapping>>,
}
#[derive(Deserialize)]
struct Mapping {
    edit: Option<Vec<Edit>>,
    position: Option<Position>,
}
#[derive(Deserialize)]
struct Edit {
    sequence: Option<String>,
    from_length: Option<usize>,
    to_length: Option<usize>,
}
#[derive(Deserialize)]
struct Position {
    offset: Option<String>,
    node_id: Option<String>,
}

/// Checks if the given Edit item is valid
fn is_valid_item(item: &Edit) -> bool {
    item.sequence.is_some() || {
        if let (Some(from_length), Some(to_length)) =
            (item.from_length, item.to_length)
        {
            from_length != to_length
        } else {
            true
        }
    }
}

/// This function processes the given position and writes the node_id to the output buffer.
fn process_position(position: &Option<Position>, output: &mut Vec<usize>) {
    if let Some(positions) = position {
        if let Some(node_id) = &positions.node_id {
            if positions.offset.is_none() {
                output.push(node_id.parse().unwrap());
            }
        }
    }
}

/// This function reads JSON values from standard input,
/// processes them, and writes the result to the output file.
pub fn run(output: String) {
    // Read from stdin
    let stdin = io::stdin();
    let handle = stdin.lock();
    let reader = io::BufReader::new(handle);

    let mut node_l: Vec<usize> = Vec::new();

    // Deserialize JSON values from the input reader into MyStruct type
    let stream =
        serde_json::Deserializer::from_reader(reader).into_iter::<MyStruct>();
    // https://github.com/vgteam/vg/issues/4202 gam format
    // Iterate over the stream of JSON values
    for value in stream {
        // Match the deserialized values
        match value {
            Ok(json) => {
                // Check if the JSON has a path with mapping
                if let Some(mapping) = json.path.and_then(|path| path.mapping) {
                    // Iterate over the mapping values
                    for i in mapping {
                        // Check if the edit field is present
                        if let Some(tem) = i.edit {
                            // Check if any item in the edit field is valid
                            if tem.iter().any(|item| is_valid_item(item)) {
                                // Continue to the next iteration
                                continue;
                            } else {
                                // Process the position and write the result to the output file
                                process_position(&i.position, &mut node_l);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to parse JSON: {}", err);
            }
        }
    }
    // count the number of nodes and repeats
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for number in node_l {
        *counts.entry(number).or_insert(0) += 1;
    }

    let path = output + ".gz";
    let file = File::create(path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());

    for (number, count) in counts {
        writeln!(encoder, "{}\t{}", number, count).unwrap();
    }

    encoder.finish().unwrap();
}
