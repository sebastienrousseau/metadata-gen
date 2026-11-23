import random, subprocess, sys, json, statistics, collections
variants = sys.argv[1].split(',')
rounds = int(sys.argv[2])
out = sys.argv[3]
root = '/tmp/hygiene-work/noyalib-bisect'
res = collections.defaultdict(lambda: collections.defaultdict(list))
for r in range(rounds):
    order = variants[:]; random.shuffle(order)
    for v in order:
        p = subprocess.run([f'{root}/{v}/target/release/examples/yb'], capture_output=True, text=True, check=True)
        for line in p.stdout.split('\n'):
            if line.strip():
                k, t = line.split(); res[k][v].append(float(t))
    json.dump(res, open(out, 'w'))
