
### Plant Genome Coordinate Conversion Workflow

#### Prerequisites
Install the following tools:
- [minimap2](https://github.com/lh3/minimap2)
- [AnchorWave](https://github.com/baoxingsong/AnchorWave)
- [TransAnno](https://github.com/informationsea/transanno)
- [Bioconvert](https://github.com/bioconvert/bioconvert)

---


#### 1. Run minimap2 to create SAM files
```bash
# Align to target genome
minimap2 -x splice -t 10 -k 12 -a -p 0.4 -N 20 Athaliana_447_TAIR10.id.chr.fa cds.fa > cds.sam

# Align to reference genome
minimap2 -x splice -t 10 -k 12 -a -p 0.4 -N 20 Col-CEN_v1.2.fastaa cds.fa > ref.sam
```

#### 2. Generate MAF files with AnchorWave
```bash
anchorwave genoAli \
  -i Col-CEN_v1.2_genes.tair10.gff3 \
  -as cds.fa \
  -r Col-CEN_v1.2.fastaa \
  -a cds.sam \
  -ar ref.sam \
  -s Athaliana_447_TAIR10.id.chr.fa \
  -n anchors \
  -o anchorwave.maf \
  -f anchorwave.f.maf \
  -IV \
  -t 10
```

#### 3. Convert MAF to Chain format
```bash
# MAF → SAM
bioconvert maf2sam anchorwave.maf anchorwave.sam

# SAM → PAF
bioconvert sam2paf anchorwave.sam anchorwave.bioconvert.paf

# PAF → Chain
transanno minimap2chain anchorwave.bioconvert.paf --output anchorwave.chain
```

---


#### 4. Run TransAnno for coordinate conversion
```bash
transanno liftbed -c anchorwave.chain 1.bed -o output.bed
```

#### input and output:
```bash
$ head 1.bed output.bed 
==> 1.bed <==
Chr1	24338315	24338316	1.929956e-08
Chr1	24339512	24339513	4.212478e-08
Chr1	24339530	24339531	1.829492e-08
Chr1	24339560	24339561	3.213491e-10
Chr1	24339970	24339971	1.254491e-08

==> output.bed <==
Chr1	26440681	26440682	1.929956e-08
Chr1	26441878	26441879	4.212478e-08
Chr1	26441896	26441897	1.829492e-08
Chr1	26441926	26441927	3.213491e-10
Chr1	26442336	26442337	1.254491e-08
```
