use bstr::io::BufReadExt;
use nohash::{BuildNoHashHasher, NoHashHasher};
use std::io::{BufReader, Write};
use std::{collections::HashMap, hash::BuildHasherDefault};

pub fn u8_slice_to_usize(slice: &[u8]) -> usize {
    let mut num: usize = 0;

    for &b in slice {
        num = num * 10 + (b - b'0') as usize;
    }
    num
}

pub struct Walk {
    pub sample: String,
    pub haptype: String,
    pub chroms: String,
    pub unit: Vec<u8>,
}
pub fn run(gfa: String, reg: String, out: String) {
    // open gfa file and read line by line
    let f = std::fs::File::open(&gfa).expect("unable to open file");
    let reader = BufReader::new(f);
    // let mut node_length = HashMap::with_capacity(100000);
    let mut node_length: HashMap<
        usize,
        u32,
        BuildHasherDefault<NoHashHasher<usize>>,
    > = HashMap::with_capacity_and_hasher(100000, BuildNoHashHasher::default());
    let mut walks: Vec<Walk> = Vec::new();
    for line in reader.byte_lines() {
        let line = line.unwrap();
        if line.starts_with(b"S") {
            let mut fields = line.split(|&b| b == b'\t');
            let _s = fields.next().unwrap();
            let seq_name = fields.next().unwrap();
            let seq_len: usize = fields.next().unwrap().len();
            let seq_name_usize = u8_slice_to_usize(seq_name);
            let seq_len_u32 = seq_len as u32;
            node_length.insert(seq_name_usize, seq_len_u32);
        } else if line.starts_with(b"W") {
            // deal with walks
            let mut fields = line.split(|&b| b == b'\t');
            let _w = fields.next().unwrap();
            let sample = String::from_utf8(fields.next().unwrap().to_vec())
                .to_owned()
                .expect("Failed to convert from UTF-8");
            // just skip if walk do nut belong to reference genome(reg)
            if sample.contains(&reg) {
                let haptype: String =
                    String::from_utf8(fields.next().unwrap().to_vec())
                        .to_owned()
                        .expect("Failed to convert from UTF-8");
                let chr: String =
                    String::from_utf8(fields.next().unwrap().to_vec())
                        .to_owned()
                        .expect("Failed to convert from UTF-8");
                let _a = fields.next().unwrap();
                let _b = fields.next().unwrap();
                let unit: Vec<u8> = fields.next().unwrap().to_vec();
                walks.push(Walk {
                    sample: sample,
                    haptype: haptype,
                    chroms: chr,
                    unit: unit,
                })
            } else {
                continue;
            }
        } else {
            continue;
        }
    }
    log::info!("Number of walks: {}", walks.len());
    // output reference genome node positions
    let output1 =
        std::fs::File::create(format!("{}.node.positions", reg)).unwrap();
    let mut writer1 = std::io::BufWriter::new(output1);
    // let mut ref_genome_node = HashMap::with_capacity(100000);
    let mut ref_genome_node: HashMap<
        usize,
        u32,
        BuildHasherDefault<NoHashHasher<usize>>,
    > = HashMap::with_capacity_and_hasher(100000, BuildNoHashHasher::default());
    for walk in walks {
        let mut sum_value: u32 = 0;
        // 直接按非数字字符分割 unit 字段
        // deepseek
        for chunk in walk.unit.split(|&b| !b.is_ascii_digit()) {
            if !chunk.is_empty() {
                let number = u8_slice_to_usize(chunk);
                // 记录节点到参考基因组的位置
                ref_genome_node.insert(number, sum_value);
                // 写入节点位置信息
                writeln!(
                    writer1,
                    "{}\t{}\t{}\t{}",
                    walk.sample, walk.chroms, number, sum_value
                )
                .unwrap();
                // 累加节点长度
                sum_value += node_length[&number];
            }
        }
    }
    writer1.flush().unwrap();
    // next step is to lift over the positions of each sample to the reference genome
    // open gfa file and read line by line
    let f = std::fs::File::open(&gfa).expect("unable to open file");
    let reader = BufReader::new(f);
    // output not reference genome node positions
    let output2 =
        std::fs::File::create(format!("{}.non_ref.node.positions", &out))
            .unwrap();
    let mut writer2 = std::io::BufWriter::new(output2);
    // bubble positions
    let output3 =
        std::fs::File::create(format!("{}.bubble.positions", &out)).unwrap();
    let mut writer3 = std::io::BufWriter::new(output3);
    for line in reader.byte_lines() {
        let line = line.unwrap();
        if line.starts_with(b"W") {
            let mut parts = line.split(|&b| b == b'\t');
            let sample =
                String::from_utf8(parts.nth(1).unwrap().to_vec()).unwrap();
            if sample.contains(&reg) {
                continue;
            } // 跳过参考基因组样本

            let chromosome =
                String::from_utf8(parts.nth(1).unwrap().to_vec()).unwrap();
            let unit_field = parts.nth(2).unwrap(); // 直接获取 unit 字段的字节序列

            // 用 split 分割数字块
            let mut bubble: Vec<usize> = Vec::new();
            for chunk in unit_field.split(|&b| !b.is_ascii_digit()) {
                if chunk.is_empty() {
                    continue;
                }

                // 解析节点 ID
                let number = u8_slice_to_usize(chunk);

                // 气泡检测逻辑
                if ref_genome_node.contains_key(&number) {
                    // 当前节点是参考节点
                    if bubble.is_empty()
                        || (bubble.len() == 1
                            && ref_genome_node.contains_key(&bubble[0]))
                    {
                        // 重置气泡，开始新的气泡（参考节点作为边界）
                        bubble = vec![number];
                    } else {
                        // 检测到完整的气泡（参考节点作为右边界）
                        let left_ref = bubble[0];
                        let left_pos = ref_genome_node
                            .get(&left_ref).copied().unwrap_or(0);

                        let right_ref = number;
                        let right_pos = ref_genome_node[&right_ref];

                        // 写入气泡元数据
                        writeln!(
                            writer3,
                            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                            chromosome,
                            sample,
                            left_ref,
                            right_ref,
                            left_pos,
                            right_pos,
                            bubble
                                .iter()
                                .skip(1)
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(",") // 跳过左参考节点
                        )
                        .unwrap();

                        // 写入非参考节点
                        for node in &bubble[1..] {
                            // 跳过左参考节点
                            if !ref_genome_node.contains_key(node) {
                                writeln!(
                                    writer2,
                                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                                    chromosome,
                                    sample,
                                    node,
                                    left_ref,
                                    right_ref,
                                    left_pos,
                                    right_pos
                                )
                                .unwrap();
                            }
                        }

                        // 重置气泡，当前参考节点作为新气泡的左边界
                        bubble = vec![number];
                    }
                } else {
                    // 非参考节点，加入气泡
                    bubble.push(number);
                }
            }

            // 处理末尾未闭合的气泡（例如：路径以非参考节点结尾）
            if bubble.len() > 1 {
                let left_ref = if ref_genome_node.contains_key(&bubble[0]) {
                    bubble[0]
                } else {
                    0 // 无效标记
                };
                let left_pos =
                    ref_genome_node.get(&left_ref).copied().unwrap_or(0);

                // 写入未闭合气泡（右边界为0）
                writeln!(
                    writer3,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    chromosome,
                    sample,
                    left_ref,
                    0, // 右参考节点标记为0
                    left_pos,
                    0, // 右位置标记为0
                    bubble
                        .iter()
                        .skip(1)
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
                .unwrap();

                // 写入非参考节点
                for node in &bubble {
                    if !ref_genome_node.contains_key(node) {
                        writeln!(
                            writer2,
                            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                            chromosome, sample, node, left_ref, 0, left_pos, 0
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    writer2.flush().unwrap();
    writer3.flush().unwrap();
}
