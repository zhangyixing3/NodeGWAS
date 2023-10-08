# **Pan_gwas**
GWAS using nodes in pan-genomics graph
## install ##
```
git clone git@github.com:zhangyixing3/pan_gwas.git
cargo build -j 2 --release
```

## &bull; Prepare merge_sample.txt file
>Map the resequencing data to the graph pan-genome, then extract the coverage information for each node. 0 represents that the sample does not contain this node.

path file (\t Separator)
```
sample1.nodes-file-path    sample1.name
sample2.nodes-file-path    sample2.name
```
```
kgwas merge -i paths -o merge_sample.txt
```
merge_sample.txt
|       | sample1 | sample2 | sample3 | sample4 | sample5 | sample6 |
|-------|---------|---------|---------|---------|---------|---------|
| node1 | 0       | 0       | 1       | 1       | 1       | 0       |
| node2 | 1       | 0       | 1       | 1       | 1       | 0       |
| node3 | 0       | 1       | 1       | 1       | 0       | 0       |
| node4 | 0       | 0       | 0       | 1       | 1       | 0       |
| node5 | 0       | 0       | 1       | 1       | 0       | 0       |



