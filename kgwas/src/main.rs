use chrono::Local;
use clap::Parser;
use env_logger::fmt::Target;
use env_logger::Builder;
use log::LevelFilter;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
mod merge;

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

#[derive(Parser, Debug)]
#[allow(non_camel_case_types)]
enum Subcli {
    /// Combine nodes files from multiple samples
    merge {
        /// input files
        #[arg(short = 'i', long = "intput", required = true)]
        input: String,
        /// prefix of output file
        #[arg(short = 'o', long = "output", default_value = "merge")]
        prefix: String,
    },
    /// to do
    filter {
        /// merge result
        #[arg(short = 'i', long = "intput", required = true)]
        input: String,
        /// prefix of output file
        #[arg(short = 'o', long = "output", default_value = "filter")]
        prefix: String,
    },
    /// kmers_table to vcf
    tovcf {
        /// kmers_table
        #[arg(short = 'k', long = "ktable", required = true)]
        ktable: String,
        /// prefix of output file
        #[arg(short = 'o', long = "output", default_value = "table")]
        prefix: String,
    },
}

#[warn(unreachable_patterns)]
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
                // Local::now().format("%Y/%m/%d %H:%M:%S"),
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
            let (a, header) = merge::filetovec(input);
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
        Subcli::tovcf { ktable, prefix } => {
            let header = "\
##fileformat=VCFv4.2
##source=kgwasV1.90
##contig=<ID=0>
##INFO=<ID=PR,Number=0,Type=Flag,Description=\"Provisional reference allele, may not be based on real reference genome\">
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n";
            let header1 = "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT";
            let output_file = format!("{}.vcf", prefix);
            let mut file = File::create(output_file).expect("create output fail");
            log::info!("Output vcf created successfully!");
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
            log::info!("VCF header has been written to the file successfully.");

            log::info!("Open kmer_table again.");
            let f: File = File::open(ktable).expect("open sample file error");
            let reader = BufReader::new(f);
            let second_reader = reader.lines().skip(1);
            log::info!("kmer_table convert to vcf ...");
            for line in second_reader {
                if let Ok(li) = line {
                    let values: Vec<&str> = li.split_whitespace().collect();
                    let chrom = "0";
                    let pos = "0";
                    let id = values[0];
                    let ref_allele = "1";
                    let alt_allele = ".";
                    let qual = ".";
                    let filter = ".";
                    let info = "PR";
                    let format_field = "GT";
                    let new_line = format!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
                        chrom, pos, id, ref_allele, alt_allele, qual, filter, info, format_field
                    );
                    let mut tem_value: Vec<&str> = Vec::new();
                    for &value in values[1..].iter() {
                        let allele = match value {
                            "0" => "0/0",
                            "1" => "1/1",
                            _ => {
                                panic!("open input file error");
                            }
                        };
                        tem_value.push(allele);
                    }
                    let tem_value = tem_value.join("\t");
                    file.write_all(format!("{}{}{}", new_line, tem_value, "\n").as_bytes())
                        .unwrap();
                }
            }

            log::info!("Congratulations, it's successful!");
        }
        _ => {
            eprint!("error command!");
            std::process::exit(1);
        }
    }
}
