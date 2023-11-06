use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub fn check_path(paths: &Vec<String>) {
    for path in paths {
        let path_str = path.as_str();
        if Path::new(path_str).is_file() {
            continue;
        } else {
            eprintln!("{} no exit !", path);
            panic!("please check your input file !");
        }
    }
}

pub fn filetovec(file: String) -> (Vec<String>, Vec<String>) {
    let f = File::open(file).expect("open input file error");
    let reader = BufReader::new(f);
    let mut paths: Vec<String> = vec![];
    let mut header: Vec<String> = vec![];

    for line in reader.lines() {
        let line = line.unwrap();
        let line: Vec<&str> = line.trim().split('\t').collect();
        paths.push(line[0].to_string());
        header.push(line[1].to_string());
    }
    (paths, header)
}

pub fn filetohash(file: &String, nodes: &mut HashSet<u32>) -> HashSet<u32> {
    let f = File::open(file).expect("open sample file error");
    let mut sub_node: HashSet<u32> = HashSet::new();
    let reader = BufReader::new(f);
    for line in reader.lines() {
        if let Ok(number_str) = line {
            if let Ok(number) = number_str.trim().parse::<u32>() {
                sub_node.insert(number);
                nodes.insert(number);
            } else {
                panic!("{}", format!("check {}", file));
            }
        } else {
            panic!("{}", format!("check {}", file));
        }
    }
    sub_node
}

/// Iterate through nodes, check if the sample contains the node, assign 1 if yes, 0 if no
pub fn count_subsample(
    prefix: String,
    all_samples: Vec<HashSet<u32>>,
    nodes: HashSet<u32>,
    header: Vec<String>,
) {
    let mut file = File::create(prefix).expect("create output fail");
    // write header
    file.write_all(b"\t").unwrap();
    let header_str = header.join("\t") + "\n";
    file.write_all(header_str.as_bytes()).unwrap();
    for i in nodes {
        let mut tem_vec: Vec<String> = Vec::new();
        tem_vec.push(i.to_string());
        for j in &all_samples {
            let mut tem: u32 = 0;
            if j.contains(&i) {
                tem = 1;
            }
            tem_vec.push(tem.to_string());
        }
        let line = tem_vec.join("\t") + "\n";
        file.write_all(line.as_bytes()).unwrap();
    }
}
