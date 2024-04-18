use bed_reader::BedErrorPlus;
use bed_reader::WriteOptions;

use ndarray::Array2;
use std::io::BufReader;
use std::{fs::File, io};
/// vec<vec<f32>> -> ndarray and reversed
pub fn vec2arr(genotype: Vec<Vec<f32>>) -> Array2<f32> {
    let rows = genotype.len();
    let cols = genotype[0].len();
    let mut arr = Array2::<f32>::zeros((cols, rows));
    for i in 0..cols {
        for j in 0..rows {
            arr[[i, j]] = genotype[j][i];
        }
    }
    arr
}

/// write something to Plink bed
pub fn write2bed(
    prefix: &str,
    sample: &[String],
    snp: &[String],
    arr: Array2<f32>,
) -> Result<(), BedErrorPlus> {
    let mut a: Vec<i32> = vec![0; snp.len()];
    for i in 0..snp.len() {
        a[i] = (i + 2) as i32;
    }
    // https://docs.rs/bed-reader/0.2.34/bed_reader/struct.WriteOptions.html#method.builder
    WriteOptions::builder(prefix)
        .fid(sample)
        .iid(sample)
        .chromosome(vec!["1"; snp.len()])
        .sid(snp)
        .bp_position(a)
        .allele_1(vec!["1"; snp.len()])
        .allele_2(vec!["."; snp.len()])
        .num_threads(10)
        .write(&arr)?;
    Ok(())
}

/// from kmer_table get matrix
pub fn get_matrix(
    second_reader: std::iter::Skip<io::Lines<BufReader<File>>>,
    sample_len: usize,
    snp_id: &mut Vec<String>,
) -> Result<Vec<Vec<f32>>, io::Error> {
    let mut genotype = Vec::new();
    for line in second_reader {
        let li = line?;
        let mut genotype_tem: Vec<f32> = Vec::with_capacity(sample_len);
        let values: Vec<&str> = li.split_whitespace().collect();
        snp_id.push(values[0].to_string());

        for &value in &values[1..] {
            let v = value.parse::<f32>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e.to_string())
            })?;
            genotype_tem.push(v);
        }
        genotype.push(genotype_tem);
    }
    Ok(genotype)
}
