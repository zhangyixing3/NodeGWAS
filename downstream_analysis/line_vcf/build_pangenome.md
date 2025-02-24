1. construct pangenome using PGGB for Saccharum
```
following link:
https://github.com/sdws1983/Saccharum-pangenome/blob/main/graph_construction/genus-level/pggb/
```
2. conbine pangenome vcf and  resequence vcf files
```
bcftools merge  --threads 2 s50000.combined.smooth.final.Srufi.filtered.revised.vcf.gz \
 Srufi.V0309.filter.recode.vcf.gz   -o  Srufi.combined.sorted.vcf.gz
```
3. construct pangenome and generate index
```
vg autoindex --threads 40 --workflow giraffe -T ./ -r ~/reference/Srufi/Srufi.v20210930.chr.fasta \
-v Srufi.combined.sorted.vcf.gz -p Srufi.combined
```