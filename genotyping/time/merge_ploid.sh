#!/bin/bash

# 帮助信息
usage() {
    echo "用法: bash $0 -i <list_file> -d <fq_dir> -o <out_dir>"
    echo "  -i: 包含样本ID的列表文件 (例如 sample.p4)"
    echo "  -d: 原始 FASTQ 文件所在目录 (例如 ../fq)"
    echo "  -o: 合并后文件的输出目录"
    exit 1
}

# 默认变量
LIST_FILE=""
FQ_DIR=""
OUT_DIR="."

# 解析命令行参数
while getopts "i:d:o:h" opt; do
    case $opt in
        i) LIST_FILE=$OPTARG ;;
        d) FQ_DIR=$OPTARG ;;
        o) OUT_DIR=$OPTARG ;;
        h) usage ;;
        *) usage ;;
    esac
done

# 检查参数是否为空
if [[ -z "$LIST_FILE" || -z "$FQ_DIR" ]]; then
    usage
fi

# 创建输出目录（如果不存在）
mkdir -p "$OUT_DIR"

# 统计处理行数
count=0

echo "--------------------------------------"
echo "开始合并任务..."
echo "列表文件: $LIST_FILE"
echo "原始目录: $FQ_DIR"
echo "保存目录: $OUT_DIR"
echo "--------------------------------------"


while read -a samples; do
    [[ -z "${samples[0]}" ]] && continue
    
    ((count++))

    r1_list=""
    r2_list=""
    for id in "${samples[@]}"; do
        r1_list+="${FQ_DIR}/ath_${id}_1.fq "
        r2_list+="${FQ_DIR}/ath_${id}_2.fq "
    done
    

    ploidy=$((${#samples[@]} * 2))
    
    # 构造输出文件名：例如 ath_poly_line1_4x_1.fq

    out_prefix="${OUT_DIR}/ath_poly_s${samples[0]}_${ploidy}x"

    
    cat $r1_list > "${out_prefix}_1.fq"
    cat $r2_list > "${out_prefix}_2.fq"


done < "$LIST_FILE"
