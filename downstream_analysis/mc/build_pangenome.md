1.construct pangenome using minigraph-cactus with default parameters.
```
cactus-pangenome ./jb  genome   --outDir result  --outName 97samples  --reference 035_Col_CEN_v12   --filter 10  --vcf full clip filter   \
--giraffe --gfa  full clip filter  --gbz full clip filter  --draw  full clip  --viz  full clip  \
-odgi  full clip filter
```
2.chop the pangenome into smaller nodes using vg.
```
vg convert  -g  97samples.gfa  -p > 97samples.vg
vg mod -X  32   97samples.vg > 97samples.chopped32.vg
vg clip -d 10  -P 035_Col_CEN_v12 -m 1000  97samples.chopped32.vg | \
 vg clip -d 1 -P 035_Col_CEN_v12 - |  \
 vg clip -sS - -P 035_Col_CEN_v12 > 97samples.chopped32.d10.vg

```
3. build index for pangenome using vg.
```
vg convert -f 97samples.chopped32.d10.vg  > 97samples.chopped32.d10.gfa
vg  autoindex -p 97samples.chopped32.d10 -g 97samples.chopped32.d10.gfa -w giraffe -t 10 -M 600G
```
4. run vg giraffe for each samples.
```
vg giraffe -t 10 -p -Z 97samples.chopped32.d10.gbz -m 97samples.chopped32.d10.min -d 97samples.chopped32.d10.dist -f /6903_1.clean.fastq.gz -f 6903_2.clean.fastq.gz 1>6903.gam 2>6903.log
```
