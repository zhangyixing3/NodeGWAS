use serde::Deserialize;
use std::fs::File;
use std::io::BufWriter;
use std::io::{self, Write};

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
    item.sequence.is_some()
        || (item.from_length.is_some() ^ item.to_length.is_some())
}

/// This function processes the given position and writes the node_id to the output buffer.
fn process_position(position: &Option<Position>, output: &mut BufWriter<File>) {
    // Check if the provided position is not None
    if let Some(position) = position {
        // Check if the offset is None and the node_id is Some
        if position.offset.is_none() && position.node_id.is_some() {
            // If the conditions are met, write the node_id to the output buffer
            if let Some(node_id) = &position.node_id {
                writeln!(output, "{}", node_id).expect("write failed");
            }
        }
    }
}

/// This function reads JSON values from standard input,
/// processes them, and writes the result to the output file.
pub fn run(output: File) {
    // Read from stdin
    let stdin = io::stdin();
    let handle = stdin.lock();
    let reader = io::BufReader::new(handle);

    // Create a buffer writer for the output file
    let mut output: io::BufWriter<File> = io::BufWriter::new(output);

    // Deserialize JSON values from the input reader into MyStruct type
    let stream =
        serde_json::Deserializer::from_reader(reader).into_iter::<MyStruct>();

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
                                process_position(&i.position, &mut output);
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
}
