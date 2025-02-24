use bstr::io::BufReadExt;
use nohash::{BuildNoHashHasher, NoHashHasher};
use std::io::{BufReader, Write};
use std::{collections::HashMap, hash::BuildHasherDefault};

pub fn u8_slice_to_u32(slice: &[u8]) -> usize {
    let mut num: usize = 0;

    for &b in slice {
        num = num * 10 + (b - b'0') as usize;
    }
    num
}

pub struct Walk {
    pub sample: String,
    pub haptype: String,
    pub chroms: String,
    pub unit: Vec<u8>,
}
pub fn run(gfa: String, reg: String, out: String) {
    // open gfa file and read line by line
    let f = std::fs::File::open(&gfa).expect("unable to open file");
    let reader = BufReader::new(f);
    // let mut node_length = HashMap::with_capacity(100000);
    let mut node_length: HashMap<
        usize,
        u32,
        BuildHasherDefault<NoHashHasher<usize>>,
    > = HashMap::with_capacity_and_hasher(100000, BuildNoHashHasher::default());
    let mut walks: Vec<Walk> = Vec::new();
    for line in reader.byte_lines() {
        let line = line.unwrap();
        if line.starts_with(b"S") {
            let mut fields = line.split(|&b| b == b'\t');
            let _s = fields.next().unwrap();
            let seq_name = fields.next().unwrap();
            let seq_len: usize = fields.next().unwrap().len();
            let seq_name_usize = u8_slice_to_u32(seq_name);
            let seq_len_u32 = seq_len as u32;
            node_length.insert(seq_name_usize, seq_len_u32);
        } else if line.starts_with(b"W") {
            // deal with walks
            let mut fields = line.split(|&b| b == b'\t');
            let _w = fields.next().unwrap();
            let sample = String::from_utf8(fields.next().unwrap().to_vec())
                .to_owned()
                .expect("Failed to convert from UTF-8");
            // just skip if walk do nut belong to reference genome(reg)
            if sample.contains(&reg) {
                let haptype: String =
                    String::from_utf8(fields.next().unwrap().to_vec())
                        .to_owned()
                        .expect("Failed to convert from UTF-8");
                let chr: String =
                    String::from_utf8(fields.next().unwrap().to_vec())
                        .to_owned()
                        .expect("Failed to convert from UTF-8");
                let _a = fields.next().unwrap();
                let _b = fields.next().unwrap();
                let unit: Vec<u8> = fields.next().unwrap().to_vec();
                walks.push(Walk {
                    sample: sample,
                    haptype: haptype,
                    chroms: chr,
                    unit: unit,
                })
            } else {
                continue;
            }
        } else {
            continue;
        }
    }
    log::info!("Number of walks: {}", walks.len());
    // output reference genome node positions
    let output1 =
        std::fs::File::create(format!("{}.node.positions", reg)).unwrap();
    let mut writer1 = std::io::BufWriter::new(output1);
    let mut current_number = Vec::new();
    // let mut ref_genome_node = HashMap::with_capacity(100000);
    let mut ref_genome_node: HashMap<
        usize,
        u32,
        BuildHasherDefault<NoHashHasher<usize>>,
    > = HashMap::with_capacity_and_hasher(100000, BuildNoHashHasher::default());
    for walk in walks {
        let mut sum_value: u32 = 0;
        for byte in walk.unit {
            if byte.is_ascii_digit() {
                current_number.push(byte);
            } else {
                if !current_number.is_empty() {
                    let number = u8_slice_to_u32(&current_number);
                    ref_genome_node.insert(number, sum_value);
                    writeln!(
                        writer1,
                        "{}\t{}\t{}\t{}",
                        walk.sample, walk.chroms, number, sum_value
                    )
                    .unwrap();
                    // println!("Node id {}", number);
                    sum_value += node_length[&number];
                }
                current_number.clear();
            }
        }
        if !current_number.is_empty() {
            let number = u8_slice_to_u32(&current_number);
            writeln!(
                writer1,
                "{}\t{}\t{}\t{}",
                walk.sample, walk.chroms, number, sum_value
            )
            .unwrap();
            sum_value += node_length[&number];
        }
    }

    writer1.flush().unwrap();
    // next step is to lift over the positions of each sample to the reference genome
    // open gfa file and read line by line
    let f = std::fs::File::open(&gfa).expect("unable to open file");
    let reader = BufReader::new(f);
    // output not reference genome node positions
    let output2 =
        std::fs::File::create(format!("{}.non_ref.node.positions", &out))
            .unwrap();
    let mut writer2 = std::io::BufWriter::new(output2);
    // bubble positions
    let output3 =
        std::fs::File::create(format!("{}.bubble.positions", &out)).unwrap();
    let mut writer3 = std::io::BufWriter::new(output3);
    for line in reader.byte_lines() {
        let line = line.unwrap();
        if line.starts_with(b"W") {
            let mut parts = line.split(|&b| b == b'\t');
            let mut current_number = Vec::new();
            let mut bubble: Vec<usize> = Vec::new();
            let sample = String::from_utf8(parts.nth(1).unwrap().to_vec())
                .to_owned()
                .expect("Failed to convert from UTF-8");
            // just skip if walk do nut belong to reference genome(reg)
            if sample.contains(&reg) {
                continue;
            }

            let chromosome = String::from_utf8(parts.nth(1).unwrap().to_vec())
                .to_owned()
                .expect("Failed to convert from UTF-8");
            for byte in parts.nth(2).unwrap() {
                if byte.is_ascii_digit() {
                    current_number.push(*byte);
                } else {
                    if !current_number.is_empty() {
                        let number = u8_slice_to_u32(&current_number);
                        current_number.clear();
                        // println!("number id {}", number);
                        if ref_genome_node.contains_key(&number) {
                            if bubble.is_empty()
                                || (ref_genome_node.contains_key(&bubble[0])
                                    && bubble.len() == 1)
                            {
                                bubble = Vec::new();
                                bubble.push(number);
                                continue;
                            } else {
                                let (tem, tem_2) = if ref_genome_node
                                    .contains_key(&bubble[0])
                                {
                                    (bubble[0], ref_genome_node[&bubble[0]])
                                } else {
                                    (0, 0)
                                };
                                let nodes: String = bubble
                                    .iter()
                                    .map(|&x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join(",");
                                // println!("{}_{}_{}_{}_{}",
                                //     tem,
                                //     number,
                                //     tem_2,
                                //     ref_genome_node[&number],
                                //     nodes);
                                writeln!(
                                    writer3,
                                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                                    chromosome,
                                    sample,
                                    tem,
                                    number,
                                    tem_2,
                                    ref_genome_node[&number],
                                    nodes
                                )
                                .unwrap();
                                for i in 0_usize..bubble.len() {
                                    if ref_genome_node.contains_key(&bubble[i])
                                    {
                                        continue;
                                    }
                                    writeln!(
                                        writer2,
                                        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                                        chromosome,
                                        sample,
                                        bubble[i],
                                        tem,
                                        number,
                                        tem_2,
                                        ref_genome_node[&number]
                                    )
                                    .unwrap();
                                }
                            }
                            bubble = Vec::new();
                            bubble.push(number);
                        } else {
                            bubble.push(number);
                        }
                    }
                }
            }

            // fot the last bubble
            if bubble.len() > 1 {
                let (tem, tem_2) = if ref_genome_node.contains_key(&bubble[0]) {
                    (bubble[0], ref_genome_node[&bubble[0]])
                } else {
                    (0, 0)
                };
                let nodes: String = bubble
                    .iter()
                    .map(|&x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(
                    writer3,
                    "{}\t{}\t{}\t{}\t{}",
                    tem, 0, tem_2, 0, nodes
                )
                .unwrap();
                for i in 0_usize..bubble.len() {
                    if ref_genome_node.contains_key(&bubble[i]) {
                        continue;
                    }
                    writeln!(
                        writer2,
                        "{}\t{}\t{}\t{}\t{}",
                        bubble[i], tem, 0, tem_2, 0
                    )
                    .unwrap();
                }
            }
        } else {
            continue;
        }
    }
    writer2.flush().unwrap();
    writer3.flush().unwrap();
}
