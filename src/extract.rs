use bstr::{io::BufReadExt, ByteSlice};
use std::fs::File;
// use std::intrinsics::mir::Len;
use memchr::{memchr2_iter, memchr_iter};
use std::io::{BufReader, BufWriter, Write};

fn _split_on_tab(line: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut start = 0;
    // 1.闭包这里用move 传递所有权， 但是外部的start 一直都是0,导致最后返回一行
    // 2.闭包这里用可变引用， 导致后面的start还是用不了。
    // let parts = memchr_iter(b'\t', line).map(|end| {
    //     let part = &line[start..end];
    //     start = end + 1;
    //     part
    // });
    // parts.chain(std::iter::once(&line[start..]))

    let mut parts = Vec::with_capacity(10);
    for end in memchr_iter(b'\t', line) {
        let part = &line[start..end];
        parts.push(part);
        start = end + 1;
    }
    parts.push(&line[start..]);
    parts.into_iter()
}

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
            let mut parts = line.split(|&b| b == b'\t');
            // let mut parts = split_on_tab(line);
            let genome = parts.nth(1).unwrap();
            if genome.starts_with(b"contig") || line.starts_with(b"unitig") {
                continue;
            }
            let hap = parts.nth(1).unwrap();
            let mut haptype_number = Vec::new();
            for i in hap {
                if i.is_ascii_digit() {
                    haptype_number.push(*i);
                }
            }
            let nodes_vec = parts.nth(2).unwrap(); // node infromation ,example => >11<12>13
            let nodes_vec_1 = &nodes_vec[1..];
            let postions = memchr2_iter(b'>', b'<', &nodes_vec_1);
            let mut left = 0_usize;
            for index in postions {
                let node_s = &nodes_vec_1[left..index];
                writer.write_all(node_s).expect("node write failed");
                writer.write_all(b"\t").expect("write failed");
                writer
                    .write_all(&haptype_number)
                    .expect("haptype information write failed");
                writer.write_all(b"\n").expect("write failed");
                left = index + 1;
            }
            // last node
            let node_s = &nodes_vec_1[left..];
            writer.write_all(node_s).expect("node write failed");
            writer.write_all(b"\t").expect("write failed");
            writer
                .write_all(&haptype_number)
                .expect("haptype information write failed");
            writer.write_all(b"\n").expect("write failed");
            // let mut tem = Vec::new();
            // for (index, i) in node_s.iter().enumerate() {
            //     if i.is_ascii_digit() {
            //         tem.push(*i);
            //         if index == node_s.len() - 1 {
            //             writer.write_all(&tem).expect("write failed");
            //             writer.write_all(b"\t").expect("write failed");
            //             writer.write_all(haptype).expect("write failed");
            //             writer.write_all(b"\n").expect("write failed");
            //         }
            //     } else {
            //         if tem.len() > 0 {
            //             writer.write_all(&tem).expect("write failed");
            //             writer.write_all(b"\t").expect("write failed");
            //             writer.write_all(haptype).expect("write failed");
            //             writer.write_all(b"\n").expect("write failed");
            //             tem.clear();
            //         }
            //     }
            // }
        } else if line.starts_with(b"P") {
            let mut parts = line.split(|&b| b == b'\t');
            // let mut parts = split_on_tab(line);  // fail to split by mmchr_iter
            let hap = parts.nth(1).unwrap();
            if hap.starts_with(b"contig") || line.starts_with(b"unitig") {
                continue;
            }
            let mut haptype_number = Vec::new();
            for i in hap {
                if i.is_ascii_digit() {
                    haptype_number.push(*i);
                }
            }

            let nodes_vec = parts.nth(0).unwrap(); // node infromation ,example => 11+,12-,13+
            let position = memchr_iter(b',', nodes_vec);
            let mut left = 0_usize;
            for index in position {
                // println!("index:{},left:{}",index,left);
                let node_s = &nodes_vec[left..index - 1];
                writer.write_all(node_s).expect("node write failed");
                writer.write_all(b"\t").expect("write failed");
                writer
                    .write_all(&haptype_number)
                    .expect("haptype information write failed");
                writer.write_all(b"\n").expect("write failed");
                left = index + 1;
            }
            // last node
            let node_s = &nodes_vec[left..nodes_vec.len() - 1];
            writer.write_all(node_s).expect("node write failed");
            writer.write_all(b"\t").expect("write failed");
            writer
                .write_all(&haptype_number)
                .expect("haptype information write failed");
            writer.write_all(b"\n").expect("write failed");
            // let mut tem = Vec::new();
            // for i in node_s {
            //     if i.is_ascii_digit() {
            //         tem.push(*i);
            //     } else {
            //         if tem.len() > 0 {
            //             writer.write_all(&tem).expect("write failed");
            //             writer.write_all(b"\t").expect("write failed");
            //             writer.write_all(haptype).expect("write failed");
            //             writer.write_all(b"\n").expect("write failed");
            //             tem.clear();
            //         }
            //     }
            // }
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

        // assert_eq!(output, "11\t1\n12\t1\n13\t1\n14\t01\n15\t01\n");
        assert_eq!(output, "11\t1\n12\t1\n13\t1\n14\t01\n15\t01\n");

        Ok(())
    }
}
