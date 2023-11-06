use bed_reader::WriteOptions;
use std::fs::File;
use std::io::BufReader;

/// vec<vec<i8>> -> ndarray and reversed
pub fn vec2arr(
    genotype: Vec<Vec<i8>>,
) -> ndarray::ArrayBase<ndarray::OwnedRepr<i8>, ndarray::Dim<[usize; 2]>> {
    let rows = genotype.len();
    let cols = genotype[0].len();
    let mut arr = ndarray::Array2::<i8>::default((cols, rows));
    for i in 0..cols {
        for j in 0..rows {
            arr[[i, j]] = genotype[j][i];
        }
    }

    arr
}

/// write something to Plink bed
pub fn write2bed(
    prefix: String,
    sample: Vec<String>,
    snp: &Vec<String>,
    arr: ndarray::ArrayBase<ndarray::OwnedRepr<i8>, ndarray::Dim<[usize; 2]>>,
) {
    // https://docs.rs/bed-reader/0.2.34/bed_reader/struct.WriteOptions.html#method.builder
    WriteOptions::builder(&prefix)
        .fid(&sample)
        .iid(&sample)
        // .father(["iid23", "iid23", "iid22"])
        // .mother(["iid34", "iid34", "iid33"])
        // .sex([1, 2, 0])
        // .pheno(["red", "red", "blue"])
        .chromosome(vec!["1"; snp.len()])
        .sid(snp)
        // .cm_position([100.4, 2000.5, 4000.7, 7000.9])
        .bp_position(vec![0; snp.len()])
        .allele_1(vec!["1"; snp.len()])
        .allele_2(vec!["."; snp.len()])
        .num_threads(5)
        .write(&arr)
        .unwrap();
}

/// from kmer_table get matrix
pub fn get_matrix(
    genotype: &mut Vec<Vec<i8>>,
    second_reader: std::iter::Skip<std::io::Lines<BufReader<File>>>,
    sample_len: usize,
    snp_id: &mut Vec<String>,
) {
    for line in second_reader {
        if let Ok(li) = line {
            let mut genotype_tem: Vec<i8> = Vec::with_capacity(sample_len);
            let values: Vec<&str> = li.split_whitespace().collect();
            snp_id.push(values[0].to_string());

            for i in 1..values.len() {
                if let Ok(value) = values[i].parse::<i8>() {
                    let value = match value {
                        0_i8 => 0_i8,
                        1_i8 => 2_i8,
                        _ => {
                            panic!("Error: Strange ")
                        }
                    };
                    genotype_tem.push(value);
                } else {
                    panic!("Error: Failed to parse value as i8");
                }
            }
            genotype.push(genotype_tem);
        }
    }
}
