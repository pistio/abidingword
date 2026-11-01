import sys; sys.path.insert(0,'/root/aw-set-e/tools')
import usfm
for d in sys.argv[1:]:
    bs=usfm.load(d)
    nv=sum(1 for b in bs.values() for _ in usfm.verses(b))
    nc=sum(len(b.chapters()) for b in bs.values())
    print(d.split('/')[-1], len(bs), nc, nv)
