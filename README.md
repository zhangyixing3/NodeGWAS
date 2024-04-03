## NodeGWAS Workflow

### Overview
![Alt text](step.png)

### <span id="installation">Installation</span>

```
git clone git@github.com:zhangyixing3/NodeGWAS.git
cd NodeGWAS && cargo build  --release
```
Finally, the binary file "nodegwas" can be found in the "NodeGWAS/target/release" directory.
### 1.Input Data

The workflow begins with the input data:


```
vg filter ../Yunzhe-94-343.gam -r 0.90 -fu -m 1 -q 15 -D 999 -t 2 -x ROC22.anchorwave.xg > Yunzhe-94-343.filtered.gam
```
- **Sample.filtered.gam** : These files contain the alignment information.
- **graph** :  The graph pangenome.

### 2.Create Node Table

Next, Extract the alignment information from each sample and compile it into a table indicating the presence or absence of nodes.

- **Accessions**: accessions or samples in the study.
- **Nodes**:  nodes within the graph pangenome.

```
$ nodegwas merge -h
merge nodes files from multiple samples
Usage: nodegwas merge [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>    input files
  -o, --output <PREFIX>  output file [default: kmer_table]
  -s, --is_sort          is_sort [default: false]
  -t, --is_transpose     is_transpose [default: false]
  -h, --help             Print help

```
is_sort: If true, sort the table by nodes.The disadvantage is that it may require additional time and memory
is_transpose: If true, transpose the table.Transforms the number of alignments into 0/1 values, where 1 indicates presence of the node in the sample.


### 3.Run GWAS analysis

The node table is then used to:

- Perform an GWAS analysis to identify associations between nodes and the phenotype of interest.
- Generate a Manhattan plot which visualizes the significance of associations between nodes and the phenotype.

### 4.Return Line Coords

Finally:
- The results are Return linear coordinates.
- This step identifies specific genes or regions of interest based on the association results.
```
Nodegwas liftover  -g  ROC22.anchorwave.giraffe.gfa -o ROC22
```
**Features:**
- [x] Able to record the memory consumption and CPU time
- [x] Able to utilize multi-threading acceleration with the Rayon library
