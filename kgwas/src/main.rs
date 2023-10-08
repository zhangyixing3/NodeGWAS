use chrono::Local;
use clap::Parser;
use env_logger::fmt::Target;
use env_logger::Builder;
use log::LevelFilter;
use std::collections::HashSet;
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
            log::info!("We have iterated through all the samples");
            log::info!("Start outputting results");
            merge::count_subsample(prefix, all_samples, nodes, header);
            log::info!("Congratulations, it's successful!");
        }
        _ => {
            eprint!("error command!");
            std::process::exit(1);
        }
    }
}
