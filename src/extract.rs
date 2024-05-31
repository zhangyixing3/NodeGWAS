use bstr::{io::BufReadExt, ByteSlice};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

pub fn run(graph: &str, node: &str) {
    let f = File::open(graph).expect("open file failed");
    let node = File::create(node).expect("create output fail");
    let mut writer = BufWriter::new(node);
    let reader = BufReader::new(f);

    for line in reader.byte_lines() {
        let line = line.expect("read line failed");
        let line = line.as_bytes();
        // //skip contig and unitig path
        // if line.starts_with(b"contig") || line.starts_with(b"unitig") {
        //     continue;
        // }
        // gfa format Walk line
        if line.starts_with(b"W") {
            let line_l: Vec<&[u8]> =
                line.trim().split(|&b| b == b'\t').collect();
            if line_l[1].starts_with(b"contig") || line.starts_with(b"unitig") {
                continue;
            }
            let haptype = line_l[3];
            let mut haptype_number = Vec::new();
            for i in haptype {
                if i.is_ascii_digit() {
                    haptype_number.push(*i);
                }
            }
            let haptype = &haptype_number;
            let node_s = line_l[6]; // node infromation ,example => >11<12>13
            let mut tem = Vec::new();
            for (index, i) in node_s.iter().enumerate() {
                if i.is_ascii_digit() {
                    tem.push(*i);
                    if index == node_s.len() - 1 {
                        writer.write_all(&tem).expect("write failed");
                        writer.write_all(b"\t").expect("write failed");
                        writer.write_all(haptype).expect("write failed");
                        writer.write_all(b"\n").expect("write failed");
                    }
                } else {
                    if tem.len() > 0 {
                        writer.write_all(&tem).expect("write failed");
                        writer.write_all(b"\t").expect("write failed");
                        writer.write_all(haptype).expect("write failed");
                        writer.write_all(b"\n").expect("write failed");
                        tem.clear();
                    }
                }
            }
        } else if line.starts_with(b"P") {
            let line_l: Vec<&[u8]> =
                line.trim().split(|&b| b == b'\t').collect();
            if line_l[1].starts_with(b"contig") || line.starts_with(b"unitig") {
                continue;
            }
            let haptype = line_l[1];
            let mut haptype_number = Vec::new();
            for i in haptype {
                if i.is_ascii_digit() {
                    haptype_number.push(*i);
                }
            }
            let haptype = &haptype_number;
            let node_s = line_l[2]; // node infromation ,example => 11+,12-,13+
            let mut tem = Vec::new();
            for i in node_s {
                if i.is_ascii_digit() {
                    tem.push(*i);
                } else {
                    if tem.len() > 0 {
                        writer.write_all(&tem).expect("write failed");
                        writer.write_all(b"\t").expect("write failed");
                        writer.write_all(haptype).expect("write failed");
                        writer.write_all(b"\n").expect("write failed");
                        tem.clear();
                    }
                }
            }
        }
    }
    writer.flush().expect("flush failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use tempdir::TempDir;

    #[test]
    fn test_run_p() -> io::Result<()> {
        let temp_dir = TempDir::new("test_extract").unwrap();
        let graph_path = temp_dir.path().join("gfaph.gfa");
        let input_data = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            P\tchr1\t11+,12-,13+\t0M,0M\n\
            P\tchr01\t14+,15-\t0M,0M";
        fs::write(&graph_path, input_data)?;
        let node_path = temp_dir.path().join("w.p.node");

        run(graph_path.to_str().unwrap(), node_path.to_str().unwrap());

        let mut output = String::new();
        File::open(node_path)?.read_to_string(&mut output)?;

        assert_eq!(output, "11\t1\n12\t1\n13\t1\n14\t01\n15\t01\n");

        Ok(())
    }
    #[test]
    fn test_run_w() -> io::Result<()> {
        let temp_dir = TempDir::new("test_extract").unwrap();
        let graph_path = temp_dir.path().join("gfaph.gfa");
        let input_data = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            W\t14\t0\tchr1\t0\t18\t>11<12>13\n\
            W\t14\t0\tchr01\t0\t18\t>14<15\n";
        fs::write(&graph_path, input_data)?;
        let node_path = temp_dir.path().join("w.p.node");

        run(graph_path.to_str().unwrap(), node_path.to_str().unwrap());

        let mut output = String::new();
        File::open(node_path)?.read_to_string(&mut output)?;

        assert_eq!(output, "11\t1\n12\t1\n13\t1\n14\t01\n15\t01\n");

        Ok(())
    }
}
