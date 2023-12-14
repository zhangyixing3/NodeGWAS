# **NodeGWAS**
NodeGWAS: A Gwas Tool Utilizing Nodes in Graph Pangenome


## <span id="installation">Installation</span>
The installation and compilation of NodeGWAS are straightforward and convenient.
```
git clone git@github.com:zhangyixing3/NodeGWAS.git
cargo build -j 10  --release
```
Finally, the binary file "Nodegwas" can be found in the "NodeGWAS/target/release" directory.

# **Running NodeGWAS**

### &bull; Extracting Node information from filtered alignment results.
```
vg filter ../Yunzhe-94-343.gam -r 0.90 -fu -m 1 -q 15 -D 999 -t 2 -x ROC22.anchorwave.xg > Yunzhe-94-343.filtered.gam
vg view -aM Yunzhe-94-343.filtered.gam | Nodegwas tonode -n Yunzhe-94-343.filtered.nodes
```
>For each sample, the corresponding Node file needs to be obtained following the steps above.

### &bull;  Filter out Nodes that appear only once.
```
Nodegwas filter -n Yunzhe-94-343.filtered.nodes  -o  Yunzhe-94-343.filtered.twice.uniq.nodes
```
>For each sample, it's the same.

### &bull;  Merge the Nodes from each sample into a Nodetable.

```
$ head -n 4 sample
../node-uniq/Yunzhe-94-343.filtered.twice.uniq.nodes  Yunzhe-94-343
../node-uniq/Zhanzhe-50.filtered.twice.uniq.nodes  Zhanzhe-50
../node-uniq/Zhanzhe-80-101.filtered.twice.uniq.nodes  Zhanzhe-80-101
../node-uniq/Zhuzhe.filtered.twice.uniq.nodes  Zhuzhe

Nodegwas merge -i sample -o Nodetable
```
> sample *.nodes  sample_name      
> Note: **\t   delimiter**

### &bull;  Extract Node information and convert the Node table into VCF.
```
Nodegwas extract  -g ROC22.anchorwave.giraffe.gfa -n w.p.nodes
Nodegwas  tovcf  -k Nodetable  -n  w.p.nodes
```
> This Step will generate files such as 1_vcf,2_vcf, 3_vcf, and so on.
> Then GWAS using Plink

### &bull;  Translate the Node positions from Walk to Path.
```
Nodegwas liftover  -g  ROC22.anchorwave.giraffe.gfa -o ROC22
```
> ref_result(The positions of Nodes on the Path)  
> ROC22(translating the positions of Nodes on the Walk to the Path)

