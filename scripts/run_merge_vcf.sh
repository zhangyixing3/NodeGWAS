#!/bin/bash -x
#PBS -q workq
#PBS -j oe
#PBS -l nodes=1:ppn=10
cd $PBS_O_WORKDIR
date -R

bgzip 1_vcf
bgzip 2_vcf
bgzip 3_vcf
bgzip 4_vcf
bgzip 5_vcf
bgzip 6_vcf
bgzip 7_vcf
bgzip 8_vcf
bgzip 9_vcf
bgzip 10_vcf
python ./merge_vcf.py  1_vcf.gz  2_vcf.gz  3_vcf.gz  4_vcf.gz  5_vcf.gz  6_vcf.gz  7_vcf.gz  8_vcf.gz  9_vcf.gz 10_vcf.gz   -o merge.vcf 1>merge_vcf.log 2>&1
