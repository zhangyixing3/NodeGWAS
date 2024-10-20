## NodeGWAS Workflow

### Overview
![Workflow Diagram](step.png)

### <span id="installation">Installation</span>

```bash
git clone git@github.com:zhangyixing3/NodeGWAS.git
cd NodeGWAS && cargo build --release
```
The binary file `nodegwas` will be located in the `NodeGWAS/target/release` directory.

### 1. Input Data

The workflow begins with the following input data:

```bash
(vg filter input_gam/AH1803.gam -r 0.90 -fu -m 1 -q 15 -D 999 -t 2 -x Srufi.combined.giraffe.xg |
vg view -aM - | nodegwas count -n input_gam/AH1803.filter.node) 1>AH1803.filter.node.log 2>&1
```
- **AH1803.gam**: VG graph alignment format.
- **Srufi.combined.giraffe.xg**: The graph pangenome.
- The output file **AH1803.filter.node.gz** records the occurrence count of each node.

### 2. Create Nodetable

Next, extract alignment information from each sample and compile it into a table indicating the presence or absence of nodes.

- **Accessions**: Accessions or samples in the study.
- **Nodes**: Nodes within the graph pangenome.

```bash
$ nodegwas rmerge -i sample.list -o node_table -n 2 -t
```

Sample list format:
```
../nodes/10020.gam.filter.node.gz	10020
../nodes/10022.gam.filter.node.gz	10022
../nodes/10023.gam.filter.node.gz	10023
../nodes/1002.gam.filter.node.gz	1002
../nodes/1061.gam.filter.node.gz	1061
../nodes/1062.gam.filter.node.gz	1062
```
**Note**: The first column is the file path, and the second column is the sample ID in the output. The output file will be `node_table2.gz`. If `-t` is used, the node table will contain only two values (0, 1).

### 3. Run GWAS Analysis

1. **EMMAX**: Since the number of nodes must be less than 20,000,000, GWAS is performed by chromosome.

```bash
// Extract nodes from the graph pangenome
$ nodegwas extract -g Srufi.combined.giraffe.gfa -n w.n.node
// Convert the node table to VCF format and split by chromosome
$ nodegwas tovcf -k nodegwas2.gz -n w.p.node
```
Output files:
```
1_vcf  2_vcf  3_vcf  4_vcf  5_vcf  6_vcf  7_vcf  8_vcf  9_vcf 10_vcf merged_vcf
```

```bash
// Perform GWAS analysis
emmax-intel64 -t emmax_in_1_vcf -o GZZTF.kinship.pca.output -p GZZTF.trait.order -k merge.kinship -c merge.pca
```
- Conduct GWAS analysis to identify associations between nodes and the phenotype of interest.
- Generate a Manhattan plot to visualize the significance of these associations.

### 4. Return Line Coordinates

Finally, the results return linear coordinates. This step identifies specific genes or regions of interest based on the association results.

```bash
Nodegwas liftover -g ROC22.anchorwave.giraffe.gfa -o node.position
```
Output:
```
Connected path, source path of the node, node,node1,node2,node1_position(offset),node2_position(offset)
ref_result three columns: path, node, position(offset)
```
By utilizing this coordinate information, we can easily identify candidate genes within a specific node range.
