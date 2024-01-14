use serde_json::Deserializer;
use std::fs::File;
use std::io::{self, Write};



pub fn run(mut output: File) {
    // from stdin
    let stdin = io::stdin();
    let handle = stdin.lock();
    let reader = io::BufReader::new(handle);

    let stream =
        Deserializer::from_reader(reader).into_iter::<serde_json::Value>();
    for value in stream {
        match value {
            Ok(json) => {
                if let Some(path) = json.get("path").and_then(|p| p.as_object())
                {
                    if let Some(mapping) =
                        path.get("mapping").and_then(|m| m.as_array())
                    {
                        for map in mapping {
                            if let Some(edit) =
                                map.get("edit").and_then(|f| f.as_array())
                            {
                                // println!("{:?}", edit);
                                if edit.iter().any(|i| {
                                    i.get("sequence").is_some()
                                        || i.as_object().map_or(false, |obj| {
                                            obj.len() % 2 != 0
                                        })
                                }) {
                                    continue;
                                }
                            }
                            // println!("{:?}", map);
                            if let Some(position) =
                                map.get("position").and_then(|f| f.as_object())
                            {
                                if position.get("offset").is_some() {
                                    continue;
                                }
                                // println!("{:?}", position);
                                if let Some(node_id) = position
                                    .get("node_id")
                                    .and_then(|f| f.as_str())
                                {
                                    writeln!(output, "{}", node_id)
                                        .expect("write failed");
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to parse JSON: {}", err);
                return;
            }
        }
    }

    // Iterate each information in the JSON and checks
}
