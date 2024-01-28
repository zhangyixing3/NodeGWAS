use chrono::Local;
use clap::Parser;
use core::panic;
use env_logger::fmt::Target;
use env_logger::Builder;
use log::LevelFilter;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
mod filter;
mod gam;
mod liftover;
mod merge;
mod tobed;

#[derive(Parser, Debug)]
#[command(
    author = "Zhang Yixing",
    version = "version 1.1",
    about = "kgwas, a tool for GWAS using kmers.",
    long_about = None
)]
struct Args {
    #[clap(subcommand)]
    command: Subcli,
}

struct Data {
    id: u32,
    new_line: String,
    tem_value: String,
    source: u8,
}

#[derive(Parser, Debug)]
#[allow(non_camel_case_types)]
enum Subcli {
    /// Combine nodes files from multiple samples
    merge {
        /// input files
        #[arg(short = 'i', long = "intput", required = true)]
        input: String,
        /// output file
        #[arg(short = 'o', long = "output", default_value = "kmer_table")]
        prefix: String,
    },
    /// kmers_table to plink bed
    tobed {
        /// merge result
        #[arg(short = 'i', long = "intput", required = true)]
        input: String,
        /// output file
        #[arg(short = 'o', long = "output", default_value = "merge.bed")]
        prefix: String,
    },
    /// kmers_table to vcf
    tovcf {
        /// kmers_table
        #[arg(short = 'k', long = "ktable", required = true)]
        ktable: String,
        /// node haptype
        #[arg(short = 'n', long = "node", required = true)]
        node: String,
        // /// output file
        // #[arg(short = 'o', long = "output", default_value = "merge.vcf")]
        // prefix: String,
    },
    /// order trait file
    order {
        /// emmax_tfam
        #[arg(short = 'e', long = "emmaxtfam", required = true)]
        tfam: String,
        /// trait file
        #[arg(short = 't', long = "trait", required = true)]
        trait_f: String,
    },
    /// get node INF
    extract {
        /// graph file
        #[arg(short = 'g', long = "graph", required = true)]
        graph: String,
        /// node source(sample)
        #[arg(short = 'n', long = "node", default_value = "nodes_INF")]
        node: String,
    },
    /// gam convert to node
    tonode {
        /// nodes output
        #[arg(short = 'n', long = "node", required = true)]
        node_file: String,
    },
    /// filter nodes < 2
    filter {
        /// gam to  node file
        #[arg(short = 'n', long = "node", required = true)]
        node_file: String,
        /// output file
        #[arg(short = 'o', long = "out", required = true)]
        output: String,
    },
    /// coordinate conversion
    liftover {
        ///  walk => path, gfa file
        #[arg(short = 'g', long = "gfa", required = true)]
        gfa: String,
        /// position result
        #[arg(short = 'o', long = "out", required = true)]
        output: String,
    },
}

fn main() {
    // init log setting
    Builder::new()
        .format(|buf, record| {
            let level = { buf.default_styled_level(record.level()) };
            let mut style = buf.style();
            style.set_bold(true); //https://docs.rs/env_logger/0.10.0/env_logger/fmt/struct.Style.html
            writeln!(
                buf,
                "{}\t[{}]\t{}",
                //    Local::now().format("%Y/%m/%d %H:%M:%S"),
                style.value(Local::now().format("%Y/%m/%d %H:%M:%S")),
                level,
                style.value(record.args())
            )
        })
        .target(Target::Stdout)
        .filter(None, LevelFilter::Debug)
        .init();

    let arg: Args = Args::parse();
    match arg.command {
        Subcli::merge { input, prefix } => {
            // return file paths and kmertable header
            let (a, header) = merge::filetovec(input);
            // we check file exit.
            merge::check_path(&a);
            log::info!("We find {} samples", a.len());
            let mut all_samples: Vec<HashSet<u32>> = Vec::new();
            let mut nodes: HashSet<u32> = HashSet::new();
            for i in a.iter() {
                all_samples.push(merge::filetohash(i, &mut nodes))
            }
            log::info!("We have iterated through all the samples.");
            log::info!("Start outputting results.");
            merge::count_subsample(prefix, all_samples, nodes, header);
            log::info!("Congratulations, it's successful!");
        }
        Subcli::tovcf { ktable, node } => {
            // get node INF
            let node = File::open(node).unwrap();
            let reader = BufReader::new(node);
            let mut node_h: HashMap<String, u8> = HashMap::new();
            for line in reader.lines() {
                let line = line.unwrap();
                let tem_value: Vec<&str> =
                    line.trim().split_whitespace().collect();
                node_h.insert(
                    tem_value[0].to_owned(),
                    tem_value[1].parse::<u8>().unwrap(),
                );
            }
            log::info!("Get node information");

            log::info!("Open kmer_table");
            let f: File = File::open(&ktable).expect("open sample file error");
            let reader = BufReader::new(f);
            let mut data_vec: Vec<Data> = Vec::new();
            let second_reader = reader.lines().skip(1);
            log::info!("kmer_table convert to vcf ...");
            for line in second_reader {
                if let Ok(li) = line {
                    let values: Vec<&str> = li.split_whitespace().collect();
                    // let chrom = "1";
                    let pos = "0";
                    let id = values[0];
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
                    let node_source = node_h.get(id);
                    let source = node_source
                        .unwrap_or_else(|| panic!("Can't find node {} INF", id))
                        .to_owned();

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
            let mut grouped_data: HashMap<u8, Vec<Data>> = HashMap::new();
            for data in data_vec {
                let entry = grouped_data.entry(data.source).or_insert(vec![]);
                entry.push(data);
            }

            // for data in data_vec {
            //     let line = format!("{}{}\n", data.new_line, data.tem_value);
            //     file.write_all(line.as_bytes()).unwrap();
            // }

            log::info!("Write INF to the VCF.");
            grouped_data.par_iter().for_each(|(key, values)| {

                let filename = format!("{}_vcf",key);
                let mut file = File::create(filename).expect("Failed to create file");
                            // vcf Header
                let header = "\
##fileformat=VCFv4.2
##source=kgwasV1.90
##INFO=<ID=PR,Number=0,Type=Flag,Description=\"Provisional reference allele, may not be based on real reference genome\">
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n";
                        let header1 = "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT";
                        file.write_all(header.as_bytes()).unwrap();
                        let f: File = File::open(&ktable).expect("open sample file error");
                        let reader = BufReader::new(f);
                        let first_line = reader.lines().next();
                        if let Some(Ok(header2)) = first_line {
                            let header_ok = format!("{}{}{}", header1, header2, "\n");
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
        Subcli::order { tfam, trait_f } => {
            let trait_file =
                File::open(&trait_f).expect("open trait file fail!");
            let trait_reader = BufReader::new(trait_file);
            let mut tem: HashMap<String, String> = HashMap::new();
            for line in trait_reader.lines() {
                let line = line.unwrap();
                let parts: Vec<&str> = line.split_whitespace().collect();
                tem.insert(parts[0].to_string(), line);
            }

            let tfam_file = File::open(tfam).expect("open tfam fail !");
            let tfam_reader = BufReader::new(tfam_file);

            let mut file =
                File::create(trait_f + ".order").expect("create output fail");
            // println!("{:?}", tem);
            for line in tfam_reader.lines() {
                let line = line.unwrap();
                let parts: Vec<&str> = line.split_whitespace().collect();

                if tem.contains_key(parts[0]) {
                    file.write_all(tem[parts[0]].as_bytes()).unwrap();
                    file.write_all(&[b'\n']).unwrap();
                } else {
                    file.write_all(
                        format!("{}\tNone\tNone\n", parts[0]).as_bytes(),
                    )
                    .unwrap();
                }
            }
        }
        Subcli::tobed { input, prefix } => {
            let sample: Vec<String>;
            let mut snp_id: Vec<String> = Vec::new();
            let mut genotype: Vec<Vec<i8>> = Vec::new();

            let f: File = File::open(&input).expect("open inuput file error");
            let reader = BufReader::new(f);

            let first_line: Option<Result<String, std::io::Error>> =
                reader.lines().next();
            if let Some(Ok(sample_tem)) = first_line {
                sample = sample_tem
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else {
                panic!("Error: Failed to read header from  input");
            }

            // check sample information
            let sample_len = sample.len();
            if sample_len == 0 {
                panic!("Error: We can't find sample")
            } else {
                log::info!("Get sample information from input");
            }

            log::info!("Open {} again.", &input);
            let f: File = File::open(input).expect("open input file error");
            let reader = BufReader::new(f);
            let second_reader: std::iter::Skip<
                std::io::Lines<BufReader<File>>,
            > = reader.lines().skip(1);
            tobed::get_matrix(
                &mut genotype,
                second_reader,
                sample_len,
                &mut snp_id,
            );

            let arr: ndarray::ArrayBase<
                ndarray::OwnedRepr<i8>,
                ndarray::Dim<[usize; 2]>,
            > = tobed::vec2arr(genotype);
            log::info!("Successfully converted Vec to Array");
            log::info!("Begin write to {}...", prefix);
            tobed::write2bed(prefix, sample, &snp_id, arr);
            log::info!("Congratulations, it's successful!");
        }
        Subcli::extract { graph, node } => {
            let f = File::open(graph).expect("open file failed");
            let mut node = File::create(node).expect("create output fail");
            let mut tem: HashSet<String> = HashSet::new();
            let reader = BufReader::new(f);

            for line in reader.lines() {
                let line = line.expect("read line failed");
                // gfa format Walk line
                if line.starts_with('W') {
                    let line_l: Vec<&str> =
                        line.trim().split_whitespace().collect();
                    let haptype: &str = line_l[3];
                    let node_s: &str = line_l[6];

                    for i in node_s.split('>') {
                        if i.is_empty() {
                            continue;
                        }

                        if !tem.contains(i) {
                            let haptype = &haptype[..haptype.len() - 1];
                            writeln!(node, "{}\t{}", i, haptype)
                                .expect("write failed");
                            tem.insert(i.to_owned());
                        }
                    }
                } else if line.starts_with('P') {
                    let line_l: Vec<&str> =
                        line.trim().split_whitespace().collect();
                    let haptype: &str = line_l[1];
                    let node_s: &str = line_l[2];
                    for i in node_s.split(&['+', '-', ','][..]) {
                        if i.is_empty() {
                            continue;
                        }
                        if !tem.contains(i) {
                            let haptype = &haptype[..haptype.len() - 1];
                            writeln!(node, "{}\t{}", i, haptype)
                                .expect("write failed");
                            tem.insert(i.to_owned());
                        }
                    }
                }
            }
        }
        Subcli::tonode { node_file } => {
            log::info!("file write to {}", &node_file);
            let node_file =
                File::create(node_file).expect("create output fail");
            gam::run(node_file);
            log::info!("Congratulations, it's successful!");
        }
        Subcli::filter { node_file, output } => {
            filter::filter_ids(&node_file, &output);
        }
        Subcli::liftover { gfa, output } => {
            log::info!("open {}", &gfa);
            liftover::run(gfa, output);
            log::info!("Congratulations, it's successful!");
        }
    }
}
