use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

struct Walk {
    node: Vec<u32>,
    ref_genome: String,
    haptype: String,
}

pub fn run(gfa: String, output: String) {
    // open input node file
    let gfa = File::open(gfa).unwrap();
    let reader = BufReader::new(gfa);
    let mut ref_node_pos = HashMap::new();
    let mut node_length = HashMap::new();
    let mut walks: Vec<Walk> = Vec::new();
    let mut output1 = File::create("ref_result").unwrap();
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with('S') {
            let l: Vec<&str> = line.split_whitespace().collect();
            node_length.insert(l[1].parse::<u32>().unwrap(), l[2].len() as u32);
        } else if line.starts_with('W') {
            let line_l: Vec<&str> = line.trim().split_whitespace().collect();
            let ref_genome: &str = line_l[3];
            let node_s: &str = line_l[6];
            let haptype: &str = line_l[1];
            let mut node: Vec<u32> = Vec::new();
            for i in node_s.split('>') {
                if i.is_empty() {
                    continue;
                }
                node.push(i.parse::<u32>().unwrap())
            }
            let walk = Walk {
                node: node,
                ref_genome: ref_genome.to_string(),
                haptype: haptype.to_string(),
            };
            walks.push(walk);
        }
        // We first need to locate the node information on the reference (ref)
        else if line.starts_with('P') {
            let line_l: Vec<&str> = line.trim().split_whitespace().collect();
            let node_s: &str = line_l[2];
            let mut sum_value: u32 = 0;
            for i in node_s.split(&['+', '-', ','][..]) {
                if i.is_empty() {
                    continue;
                } else {
                    let i_u32 = i.parse::<u32>().unwrap();
                    ref_node_pos.insert(i_u32, sum_value);
                    writeln!(output1, "{}\t{}\t{}", line_l[1], i, sum_value,)
                        .unwrap();
                    sum_value += node_length[&i_u32];
                }
            }
        }
    }

    //
    let mut output = File::create(output).unwrap();

    // By Walk information to count the positions
    for i in walks {
        let haptype = i.haptype;
        let ref_genome = i.ref_genome;
        let mut head = 0;
        let mut tem_vec: Vec<u32> = Vec::with_capacity(100);
        for i in i.node {
            if ref_node_pos.contains_key(&i) {
                for ii in &tem_vec {
                    writeln!(
                        output,
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        ref_genome,
                        haptype,
                        ii,
                        head,
                        i,
                        ref_node_pos[&head],
                        ref_node_pos[&i]
                    )
                    .unwrap();
                }
                head = i;
                tem_vec.clear();
            } else {
                tem_vec.push(i);
            }
        }
    }
}
