use flate2::write::GzEncoder;
use flate2::Compression;
use memchr::memchr;
use memchr::memrchr;
use nohash::NoHashHasher;
use serde::Deserialize;
use serde_json::from_slice;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::{collections::HashMap, hash::BuildHasherDefault};

const READ_BUF_SIZE: usize = 512 * 1024; //  512 KiB
const THRED_NUM: usize = 5; // number of threads to use

#[derive(Deserialize)]
struct Path {
    mapping: Option<Vec<Mapping>>,
}
#[derive(Deserialize)]
struct FStruct {
    path: Path,
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
fn process_chunk(data: &[u8], result: &mut Vec<usize>) {
    let mut start = 0;
    while let Some(end) = memchr(b'\n', &data[start..]) {
        let line_end = start + end;
        let line = &data[start..line_end];
        start = line_end + 1; // Move past the newline character

        // // Parse JSON from line
        if let Ok(fstruct) = from_slice::<FStruct>(line) {
            for mapping in fstruct.path.mapping.iter().flatten() {
                if let Some(tem) = &mapping.edit {
                    if tem.iter().any(|item| is_valid_item(item)) {
                        continue;
                    } else {
                        process_position(&mapping.position, result);
                    }
                }
            }
        }
    }
}

fn find_new_line_pos(bytes: &[u8]) -> Option<usize> {
    // https://docs.rs/memchr/latest/memchr/fn.memrchr.html
    memrchr(b'\n', bytes)
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
    let mut file = File::open("/dev/stdin").expect("Failed to open stdin");
    // Start the processor threads
    // https://github.com/Naveenaidu/rust-1brc/blob/main/src/main.rs#L104-154
    let (sender, receiver) = crossbeam_channel::bounded::<Box<[u8]>>(1000);
    // let n_threads = std::thread::available_parallelism().unwrap().into();
    let mut handles = Vec::with_capacity(THRED_NUM);
    for _ in 0..THRED_NUM {
        let receiver = receiver.clone();
        let handle = std::thread::spawn(move || {
            let mut sub_result: Vec<usize> = Vec::with_capacity(2000);
            // wait until the sender sends the chunk
            for buf in receiver {
                process_chunk(&buf, &mut sub_result);
            }
            sub_result
        });
        handles.push(handle);
    }

    // Read the file in chunks and send the chunks to the processor threads
    let mut buf = vec![0; READ_BUF_SIZE];
    let mut bytes_not_processed = 0;
    loop {
        let bytes_read = file
            .read(&mut buf[bytes_not_processed..])
            .expect("Failed to read file");
        if bytes_read == 0 {
            break;
        }

        let actual_buf = &mut buf[..bytes_not_processed + bytes_read];
        let last_new_line_index = match find_new_line_pos(&actual_buf) {
            Some(index) => index,
            None => {
                bytes_not_processed += bytes_read;
                if bytes_not_processed == buf.len() {
                    buf.resize(buf.len() * 2, 0);
                }
                continue; // try again
            }
        };

        let buf_boxed =
            Box::<[u8]>::from(&actual_buf[..(last_new_line_index + 1)]);
        sender.send(buf_boxed).expect("Failed to send buffer");

        actual_buf.copy_within(last_new_line_index + 1.., 0);
        // You cannot use bytes_not_processed = bytes_read - last_new_line_index
        // - 1; because the buffer will contain unprocessed bytes from the
        // previous iteration and the new line index will be calculated from the
        // start of the buffer
        bytes_not_processed = actual_buf.len() - last_new_line_index - 1;
    }
    drop(sender);

    // count the number of nodes and repeats
    // use nohash instead of default hasher
    // let mut nodes: HashMap<usize, usize> = HashMap::new();
    let mut nodes: HashMap<
        usize,
        usize,
        BuildHasherDefault<NoHashHasher<usize>>,
    > = HashMap::default();
    for handle in handles {
        let sub_result = handle.join().unwrap();
        for number in sub_result {
            *nodes.entry(number).or_insert(0) += 1;
        }
    }

    let path = output + ".gz";
    let file = File::create(path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());

    for (number, count) in nodes {
        writeln!(encoder, "{}\t{}", number, count).unwrap();
    }
    encoder.finish().unwrap();
}
