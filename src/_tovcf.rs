use std::io::{self, BufRead, Read};
use std::{fs::File, io::BufReader};
use std::{collections::HashMap, hash::BuildHasherDefault};
use bstr::{io::BufReadExt, ByteSlice};
use flate2::bufread::GzDecoder;
use nohash::NoHashHasher;

struct Data {
    id: u32,
    new_line: String,
    tem_value: String,
    source: u32,
}

fn gzip_true(filepath: &str) -> io::Result<bool> {
    let mut infile = File::open(filepath)?;
    let mut buf = [0u8; 2];
    infile.read_exact(&mut buf)?;
    Ok(buf[0] == 31 && buf[1] == 139)
}

pub fn run(node: &str, ktable: &str) {
    // get node INF
    let node = File::open(node).unwrap();
    let reader = BufReader::new(node);
    let hasher = BuildHasherDefault::<NoHashHasher<usize>>::default();
    let mut nodes_infor: HashMap<usize, usize, BuildHasherDefault<NoHashHasher<usize>>> = HashMap::with_capacity_and_hasher(10000, hasher);
    for line in reader.byte_lines() {
        let line = line.unwrap();
        let pos = line.iter().position(|&b| b == b'\t').unwrap();
        let chr = line[..pos].to_str().unwrap().parse::<usize>().unwrap();
        let node = line[pos + 1..].to_str().unwrap().parse::<usize>().unwrap();
        nodes_infor.insert(node, chr);
    }
    log::info!("Node information loaded");

    log::info!("Open kmer_table");
    let node_table = File::open(&ktable).expect("Failed to open file");
    let file_reader = BufReader::new(node_table);

    let reader: BufReader<Box<dyn Read>> =
        if gzip_true(&ktable).unwrap() {
            BufReader::new(
                Box::new(GzDecoder::new(file_reader)) as Box<dyn Read>
            )
        } else {
            BufReader::new(Box::new(file_reader) as Box<dyn Read>)
        };

    log::info!("kmer_table convert to vcf ...");


    let mut data_vec: Vec<Data> = Vec::new();
    let second_reader = reader.lines().skip(1);
    log::info!("kmer_table convert to vcf ...");
    for line in second_reader {
        if let Ok(li) = line {
            let values: Vec<&str> = li.split_whitespace().collect();
            // let chrom = "1";
            let pos = "0";
            let id = values[0].to_owned().parse::<usize>().unwrap();
            let ref_allele = "1";
            let alt_allele = ".";
            let qual = ".";
            let filter = ".";
            let info = "PR";
            let format_field = "GT";
            let new_line = format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
                pos,
                id,
                ref_allele,
                alt_allele,
                qual,
                filter,
                info,
                format_field
            );
            let mut tem_value: Vec<&str> = Vec::new();
            for &value in values[1..].iter() {
                let allele = match value {
                    // kmer_table  0 = Absence,  1 = Presence
                    // vcf    0 =  Presence, 1 = Absence
                    "0" => "1/1",
                    "1" => "0/0",
                    _ => {
                        panic!("open input file error");
                    }
                };
                tem_value.push(allele);
            }
            let tem_value = tem_value.join("\t");
            let node_source = nodes_infor.get(&id);
            let mut source = 0_usize;
            match  node_source {
                Some(tem) => {
                    source = *tem;
                }
                None => {
                    continue;
                }
            }
            // let source = node_source
            //     .unwrap_or_else(|| panic!("Can't find node {} INF", id))
            //     .to_owned();

            let data: Data = Data {
                id: id.parse::<u32>().unwrap(),
                new_line: format!("{}\t{}", source, new_line),
                tem_value,
                source,
            };

            data_vec.push(data);
        }
    }

    // Sort the data by id(nodes)
    data_vec.par_sort_unstable_by(|a, b| a.id.cmp(&b.id));
    let mut grouped_data: HashMap<u32, Vec<Data>> = HashMap::new();
    for data in data_vec {
        let entry = grouped_data.entry(data.source).or_insert(vec![]);
        entry.push(data);
    }

    log::info!("Write INF to the VCF.");
    grouped_data.par_iter().for_each(|(key, values)| {

        let filename = format!("{}_vcf",key);
        let mut file = File::create(filename).expect("Failed to create file");
                    // vcf Header
        let header = b"\
            ##fileformat=VCFv4.2\n\
            ##source=kgwasV1.90\n\
            ##INFO=<ID=PR,Number=0,Type=Flag,Description=\"Provisional reference allele,\
            may not be based on real reference genome\"\n\
            ##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\"\n";
                let header1 = "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT";
                file.write_all(header).unwrap();
                let node_f = File::open(&ktable).expect("Failed to open file");
                let file_reader = BufReader::new(node_f);
                let reader: BufReader<Box<dyn Read>> = if gzip_true(&ktable).unwrap() {
                    BufReader::new(Box::new(GzDecoder::new(file_reader)) as Box<dyn Read>)
                } else {
                    BufReader::new(Box::new(file_reader) as Box<dyn Read>)
                };
                let first_line = reader.lines().next();
                if let Some(Ok(header2)) = first_line {
                    let header_ok = format!("{}{}{}", header1, &header2[5..], "\n");
                    file.write_all(header_ok.as_bytes()).unwrap();
                } else {
                    panic!("Error: Failed to read header2 from the file.");
                }

        // 将 values 中的数据写入文件
        for data in values {
            let line = format!("{}{}\n", data.new_line, data.tem_value);
            file.write_all(line.as_bytes()).expect("Failed to write to file");
        }
});

    log::info!("Congratulations, it's successful!");
}
