### simulate ara genomes
```
for i in `seq 1 65`;do  echo -e "varsim.py --simulator_executable    art_illumina    --id ath_${i} --seed $i --reference  Chr1.fa   --vc_num_snp 100000 --vc_num_ins 10000 --vc_num_del 10000 --vc_num_mnp 1000 --vc_num_complex 2500 --vc_min_length_lim 0 --vc_max_length_lim 100000 --disable_sim --vc_prop_het 0.6 --vc_in_vcf Chr01.vcf.gz  --disable_rand_dgv --out_dir out_sim_${i} --log_dir log_sim_${i} --work_dir work_sim_${i}"; done 
```
### simulate ngs data
```
for i in ./fa_dir/*fa; do a=$(basename $i .fa );echo "art_illumina -p -na -l 150 -ss HS25 -i  ${i} -f 10 -m 500 -s 10 -o ./${a}_";done 
```

### merge diploid genome
```
generate  list.p2 list.p4 list.p6 list.p8
python random1.py
awk '{print "cat " ../fasta/ath_" $1 ".fa" " ../fasta/ath_" $2 ".fa" " ../fasta/ath_" $3 ".fa" " ../fasta/ath_" $4 ".fa" " ../fasta/ath_" $5 ".fa" > fasta/simp10." NR ".fa"}' list.p10 | bash -
awk '{print "cat " ../fasta/ath_" $1 ".fa" " ../fasta/ath_" $2 ".fa" " ../fasta/ath_" $3 ".fa" " ../fasta/ath_" $4 ".fa" " ../fasta/ath_" $5 ".fa" " ../fasta/ath_" $6 ".fa" > fasta/simp12." NR ".fa"}' list.p12 | bash -
awk '{print "cat " ../fasta/ath_" $1 ".fa" " ../fasta/ath_" $2 ".fa" " ../fasta/ath_" $3 ".fa" " ../fasta/ath_" $4 ".fa" " ../fasta/ath_" $5 ".fa" " ../fasta/ath_" $6 ".fa" " ../fasta/ath_" $7 ".fa" > fasta/simp14." NR ".fa"}' list.p14 | bash -
awk '{print "cat " ../fasta/ath_" $1 ".fa" " ../fasta/ath_" $2 ".fa" > fasta/simp4." NR ".fa"}' list.p4 | bash -
awk '{print "cat " ../fasta/ath_" $1 ".fa" " ../fasta/ath_" $2 ".fa" " ../fasta/ath_" $3 ".fa" > fasta/simp6." NR ".fa"}' list.p6 | bash -
awk '{print "cat " ../fasta/ath_" $1 ".fa" " ../fasta/ath_" $2 ".fa" " ../fasta/ath_" $3 ".fa" " ../fasta/ath_" $4 ".fa" > fasta/simp8." NR ".fa"}' list.p8 | bash -
awk '{print "bcftools merge -0 -m all --info-rules - --use-header header  ../vcfs/ath_$1.truth.vcf.gz ../vcfs/ath_$2.truth.vcf.gz ../vcfs/ath_$3.truth.vcf.gz ../vcfs/ath_$4.truth.vcf.gz ../vcfs/ath_$5.truth.vcf.gz | bgzip -@ 24 -c > vcfs/merged_p10."NR".vcf.gz"}' list.p10 > merge_p10.sh
awk '{print "bcftools merge -0 -m all --info-rules - --use-header header  ../vcfs/ath_$1.truth.vcf.gz ../vcfs/ath_$2.truth.vcf.gz ../vcfs/ath_$3.truth.vcf.gz ../vcfs/ath_$4.truth.vcf.gz ../vcfs/ath_$5.truth.vcf.gz ../vcfs/ath_$6.truth.vcf.gz | bgzip -@ 24 -c > vcfs/merged_p12."NR".vcf.gz"}' list.p12 > merge_p12.sh
awk '{print "bcftools merge -0 -m all --info-rules - --use-header header  ../vcfs/ath_$1.truth.vcf.gz ../vcfs/ath_$2.truth.vcf.gz ../vcfs/ath_$3.truth.vcf.gz ../vcfs/ath_$4.truth.vcf.gz ../vcfs/ath_$5.truth.vcf.gz ../vcfs/ath_$6.truth.vcf.gz ../vcfs/ath_$7.truth.vcf.gz | bgzip -@ 24 -c > vcfs/merged_p14."NR".vcf.gz"}' list.p14 > merge_p14.sh
awk '{print "bcftools merge -0 -m all --info-rules - --use-header header  ../vcfs/ath_$1.truth.vcf.gz ../vcfs/ath_$2.truth.vcf.gz | bgzip -@ 24 -c > vcfs/merged_p4."NR".vcf.gz"}' list.p4 > merge_p4.sh
awk '{print "bcftools merge -0 -m all --info-rules - --use-header header  ../vcfs/ath_$1.truth.vcf.gz ../vcfs/ath_$2.truth.vcf.gz ../vcfs/ath_$3.truth.vcf.gz | bgzip -@ 24 -c > vcfs/merged_p6."NR".vcf.gz"}' list.p6 > merge_p6.sh
awk '{print "bcftools merge -0 -m all --info-rules - --use-header header  ../vcfs/ath_$1.truth.vcf.gz ../vcfs/ath_$2.truth.vcf.gz ../vcfs/ath_$3.truth.vcf.gz ../vcfs/ath_$4.truth.vcf.gz | bgzip -@ 24 -c > vcfs/merged_p8."NR".vcf.gz"}' list.p8 > merge_p8.sh
for i in merged*vcf.gz; do python ./vcf_flatten.py  ${i} | bgzip > polyploid_${i}; done
```

### call variants
```
fastp -i ~/2025/major_revise_round1/simulate_polyploid/simulate_raw_reads/10x/ath_8_1.fq -I ~/2025/major_revise_round1/simulate_polyploid/simulate_raw_reads/10x/ath_8_2.fq -o ath_8_1.fq -O ath_8_2.fq
fastp -i ~/2025/major_revise_round1/simulate_polyploid/simulate_raw_reads/10x/ath_9_1.fq -I ~/2025/major_revise_round1/simulate_polyploid/simulate_raw_reads/10x/ath_9_2.fq -o ath_9_1.fq -O ath_9_2.fq
bwa mem -t 10 -R "@RG\tID:ath_8\tSM:ath_8\tPL:illumina\tLB:ath_8" reffa/Chr1.fa ath_8_1.fq  ath_8_2.fq    | samtools view -@ 10 -bS -q 20 - | samtools sort -@ 10 -T ath_8         -o ./ath_8.q20.sorted.bam -  
bwa mem -t 10 -R "@RG\tID:ath_9\tSM:ath_9\tPL:illumina\tLB:ath_9" reffa/Chr1.fa ath_9_1.fq  ath_9_2.fq    | samtools view -@ 10 -bS -q 20 - | samtools sort -@ 10 -T ath_9         -o ./ath_9.q20.sorted.bam - 
## 10X ,then reheader
samtools merge -@ 10 p10/merged_p10_32.bam ath_32.q20.sorted.bam        ath_43.q20.sorted.bam   ath_62.q20.sorted.bam   ath_64.q20.sorted.bam    ath_65.q20.sorted.bam
## 8X ,then reheader
samtools merge -@ 10 p8/merged_p8_1.bam ath_34.q20.sorted.bam   ath_12.q20.sorted.bam   ath_14.q20.sorted.bam   ath_48.q20.sorted.bam
......
GATK Pipeline

```

### evaluate  pricision
```
```


```
