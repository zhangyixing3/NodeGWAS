# NodeGWAS Workflow

## Overview
![Alt text](step.png)

## 1.Input Data

The workflow begins with the input data:


```
vg filter ../Yunzhe-94-343.gam -r 0.90 -fu -m 1 -q 15 -D 999 -t 2 -x ROC22.anchorwave.xg > Yunzhe-94-343.filtered.gam
```
- **.filtered.gam** : These files contain the alignment information.
- **graph** :  The graph pangenome.

## 2.Create Node Table

Next, Extract the alignment information from each sample and compile it into a table indicating the presence or absence of nodes.

- **Accessions**: accessions or samples in the study.
- **Nodes**:  nodes within the graph pangenome.
- The table represents the presence or absence of nodes in the population.
```
to do
```


## 3.Run GWAS analysis

The node table is then used to:

- Perform an GWAS analysis to identify associations between nodes and the phenotype of interest.
- Generate a Manhattan plot which visualizes the significance of associations between nodes and the phenotype.

## 4.Return Line Coords

Finally:
- The results are Return linear coordinates.
- This step identifies specific genes or regions of interest based on the association results.
```
Nodegwas liftover  -g  ROC22.anchorwave.giraffe.gfa -o ROC22
```
