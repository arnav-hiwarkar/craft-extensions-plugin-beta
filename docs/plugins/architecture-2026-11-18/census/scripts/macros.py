import re,sys,os,glob
root=sys.argv[1]
for f in glob.glob(root+'/**/*.rs',recursive=True):
    if '/target/' in f or '/tests/' in f: continue
    s=open(f,errors='ignore').read()
    for m in re.finditer(r'macro_rules!\s*(\w+)\s*\{',s):
        # find body to matching brace
        i=m.end(); d=1
        while d and i<len(s):
            if s[i]=='{': d+=1
            elif s[i]=='}': d-=1
            i+=1
        body=s[m.end():i]
        if re.search(r'CommandSpec|\bcmd!|\bquery!|\bspec!|\bstub!',body):
            print(f.replace(root+'/',''), m.group(1), len(body))
