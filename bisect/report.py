import json, sys, statistics
res = json.load(open(sys.argv[1])); ref = sys.argv[2]
for k, d in res.items():
    rm = statistics.median(d[ref])
    print(f"== {k} (ref {ref} median {rm:.0f} ns, n={len(d[ref])})")
    for v, xs in d.items():
        m = statistics.median(xs); q = statistics.quantiles(xs, n=4) if len(xs)>3 else [min(xs),m,max(xs)]
        print(f"  {v:6s} med {m:12.1f}  IQR [{q[0]:.0f},{q[2]:.0f}]  min {min(xs):.0f}  delta {100*(m/rm-1):+5.1f}%")
