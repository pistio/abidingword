import sys,re,glob,os
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
import usfm
for d in sys.argv[1:]:
    tot=0;raw=0;empty=0
    for f in sorted(glob.glob(d+'/*.usfm')):
        t=open(f,encoding='utf-8-sig').read()
        b=usfm.parse_usfm(t)
        vs=usfm.verses_of(b)
        tot+=len(vs);raw+=len(re.findall(r'\\v ',t))
        e=[(c,v) for c,v,x in vs if not x]
        empty+=len(e)
        if e: print('  empty',b['code'],e[:5])
        styles=set(bl['style'] for bl in b['blocks'] if bl['kind']=='para')
    print(d,tot,raw,'empty',empty)
