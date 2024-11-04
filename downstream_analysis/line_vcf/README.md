## `line + vcf` downstream analysis workflow
### 1. Merge all `.ps` files
```
ls */GZZTF*.ps
10/GZZTF.result.ps  2/GZZTF.result.ps  4/GZZTF.result.ps  6/GZZTF.result.ps  8/GZZTF.result.ps
1/GZZTF.result.ps   3/GZZTF.result.ps  5/GZZTF.result.ps  7/GZZTF.result.ps  9/GZZTF.result.ps
bash PATH/scripts/merge_ps.sh "*/GZZTF.result.ps" > GZZTF.result.ps
```

### 2. Filter significant sites
```
awk '$4 <= 1/45592180 {print $0}' GZZTF.result.ps > GZZTF.result.ps.1_sigSite.out
```

### 3. Return Line Coordinates
```
nodegwas liftover -g Srufi.anchorwave.giraffe.gfa -o node
head node ref_result
==> node <==
#ref_path    path    nodeid    node1    node2    offset1    offset2
Chr02    ROC22_D    23566711    23566710    23566713    327868    327882
Chr02    ROC22_D    23566728    23566727    23566729    327936    327937
Chr02    ROC22_D    23566733    23566732    23566735    327946    327948
Chr02    ROC22_D    23566736    23566735    23566738    327948    327954
Chr02    ROC22_D    23566749    23566744    23566750    327967    327968
Chr02    ROC22_D    23566789    23566788    23566791    328062    328072
Chr02    ROC22_D    23566794    23566791    23566795    328072    328075
Chr02    ROC22_D    23566798    23566797    23566804    328078    328081
Chr02    ROC22_D    23566799    23566797    23566804    328078    328081
Chr02    ROC22_D    23566807    23566804    23566808    328081    328083

==> ref_result <==
#path    nodeid    offset
contig_000538    158605794    0
contig_000538    158605795    32
contig_000538    158605796    64
contig_000538    158605797    96
contig_000538    158605798    128
contig_000538    158605799    160
contig_000538    158605800    192
contig_000538    158605801    224
contig_000538    158605802    256
contig_000538    158605803    288
```

### 4. Find the positions of important nodes
```
awk 'NR==FNR { set[$1]; next } $3 in set { print }' GZZTF.result.ps.1_sigSite.out node > GZZTF.result.ps.1_sigSite.out_node
awk 'NR==FNR { set[$1]; next } $2 in set { print }' GZZTF.result.ps.1_sigSite.out ref_result > GZZTF.result.ps.1_sigSite.out_ref_result
```

### 5. Find the range of important sites
```
awk '{print $1 "\t" $6-5000 "\t" $7+5000}' GZZTF.result.ps.1_sigSite.out_node > GZZTF.result.ps.1_sigSite.out_node_range
awk '{print $1 "\t" $3-5000 "\t" $3+5000}' GZZTF.result.ps.1_sigSite.out_ref_result > GZZTF.result.ps.1_sigSite.out_ref_result_range
```

### 6. Combine the two range files
```
cat GZZTF.result.ps.1_sigSite.out_node_range GZZTF.result.ps.1_sigSite.out_ref_result_range > pos.bubble.positions.TEM.chr.merge
```

### 7. Use Bedtools to find overlapping ranges
```
bedtools intersect -a pos.bubble.positions.TEM.chr.merge -b Srufi.chr.bed -wb > positions.TEM.chr.merge.overlap
```

