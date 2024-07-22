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
(vg filter input_gam/AH1803.gam -r 0.90 -fu -m 1 -q 15 -D 999 -t 2 -x Srufi.combined.giraffe.xg  |
vg view -aM - | nodegwas count -n input_gam/AH1803.filter.node) 1>AH1803.filter.node.log 2>&1
```
- **AH1803.gam** : vg graph alignment format
- **Srufi.combined.giraffe.xg** :  The graph pangenome.
- The output file **AH1803.filter.node.gz** records the occurrence count of each node

### 2.Create Node Table

Next, Extract the alignment information from each sample and compile it into a table indicating the presence or absence of nodes.

- **Accessions**: accessions or samples in the study.
- **Nodes**:  nodes within the graph pangenome.

```
$ nodegwas rmerge -i sample.list -o node_table -n 2 -t
$ head sample.list
../nodes/10020.gam.filter.node.gz	10020
../nodes/10022.gam.filter.node.gz	10022
../nodes/10023.gam.filter.node.gz	10023
../nodes/1002.gam.filter.node.gz	1002
../nodes/1061.gam.filter.node.gz	1061
../nodes/1062.gam.filter.node.gz	1062
```
**Note** : The first column is file path, the second column is sample id in output.

if use -t , The node_table only contains two number(0, 1).

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
- [x] Able to use multi-threading acceleration with the Rayon library
