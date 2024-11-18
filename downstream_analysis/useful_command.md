```bash
ls ~/stu_zhangyixing/Nodegwas/ara/d10/d10/*gz | awk -F/ '{print $0, $(NF)}' | awk '{split($2, a, "."); print $1, a[1]}' | head
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/10020.gam.filter.node.gz 10020
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/10022.gam.filter.node.gz 10022
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/10023.gam.filter.node.gz 10023
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/10027.gam.filter.node.gz 10027
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/1002.gam.filter.node.gz 1002
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/1006.gam.filter.node.gz 1006
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/1061.gam.filter.node.gz 1061
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/1062.gam.filter.node.gz 1062
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/1063.gam.filter.node.gz 1063
/public/home/off_huangyumin/stu_zhangyixing/Nodegwas/ara/d10/d10/1066.gam.filter.node.gz 1066
```
