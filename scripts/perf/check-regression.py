#!/usr/bin/env python3
import argparse,json,math,pathlib

def percentile(values,p):
    v=sorted(float(x) for x in values)
    if not v: raise ValueError("empty sample set")
    return v[max(0,math.ceil(len(v)*p)-1)]

def ranks(values):
    order=sorted(range(len(values)),key=lambda i:values[i]);r=[0.0]*len(values);i=0
    while i<len(order):
        j=i
        while j+1<len(order) and values[order[j+1]]==values[order[i]]:j+=1
        rank=(i+j+2)/2
        for k in range(i,j+1):r[order[k]]=rank
        i=j+1
    return r

def mann_whitney_one_sided_greater(base,current):
    # H1: candidate/current distribution is shifted upward (worse).
    x=[float(v) for v in base];y=[float(v) for v in current];n1=len(x);n2=len(y)
    if n1<2 or n2<2:return 1.0
    allv=x+y;rr=ranks(allv);ry=sum(rr[n1:]);u=ry-n2*(n2+1)/2
    mean=n1*n2/2
    counts={}
    for v in allv:counts[v]=counts.get(v,0)+1
    N=n1+n2
    tie=sum(t**3-t for t in counts.values())
    var=n1*n2/12*((N+1)-tie/(N*(N-1))) if N>1 else 0
    if var<=0:return 1.0 if u<=mean else 0.0
    z=(u-mean-0.5)/math.sqrt(var)
    return 0.5*math.erfc(z/math.sqrt(2))

def practical(base,current,rel,abs_):
    bp=percentile(base,.95);cp=percentile(current,.95)
    threshold=bp+max(abs_,bp*rel)
    return cp>threshold,bp,cp,threshold

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--baseline-manifest",required=True);ap.add_argument("--baseline-metrics",required=True);ap.add_argument("--baseline-samples",required=True)
    ap.add_argument("--current-metrics",required=True);ap.add_argument("--current-samples",required=True);ap.add_argument("--output",required=True)
    a=ap.parse_args()
    manifest=json.load(open(a.baseline_manifest));bm=json.load(open(a.baseline_metrics));bs=json.load(open(a.baseline_samples));cm=json.load(open(a.current_metrics));cs=json.load(open(a.current_samples))
    pol=manifest["regression_policy"];alpha=float(pol["alpha"]);failures=[];results={}
    series=[
      ("idle_cpu_percent","cpu_percent","idle_cpu_percent"),
      ("idle_rss_mib","rss_mib","idle_rss_mib"),
      ("startup_warm_ms","warm_startup_ms","startup_warm_ms"),
      ("startup_cold_ms","cold_startup_ms","startup_cold_ms")
    ]
    for name,key,pk in series:
        cfg=pol[pk];sig=mann_whitney_one_sided_greater(bs[key],cs[key])
        bad,bp,cp,threshold=practical(bs[key],cs[key],float(cfg["relative_p95_increase"]),float(cfg["absolute_p95_increase"]))
        reg=bad and sig<alpha
        results[name]={"baseline_p95":bp,"current_p95":cp,"practical_threshold":threshold,"p_value_one_sided":sig,"alpha":alpha,"regression":reg}
        if reg:failures.append(f"{name}: p95 {cp:.4f} > {threshold:.4f}, p={sig:.6g}")
    bnet=float(bm["idle_network_requests_10m"]);cnet=float(cm["idle_network_requests_10m"]);net_bad=cnet>bnet+float(pol["network_requests"]["allowed_increase"])
    results["network_requests"]={"baseline":bnet,"current":cnet,"regression":net_bad}
    if net_bad:failures.append(f"network_requests: {cnet} > baseline {bnet}")
    for key,pk in [("stripped_executable_mib","stripped_executable_mib"),("installer_mib","installer_mib")]:
        cfg=pol[pk];bv=float(bm[key]);cv=float(cm[key]);threshold=bv+max(float(cfg["absolute_increase"]),bv*float(cfg["relative_increase"]));bad=cv>threshold
        results[key]={"baseline":bv,"current":cv,"practical_threshold":threshold,"regression":bad,"method":"deterministic-size-delta"}
        if bad:failures.append(f"{key}: {cv:.4f} > {threshold:.4f}")
    report={"schema":1,"baseline_commit":manifest["baseline_commit"],"alpha":alpha,"results":results,"failures":failures,"pass":not failures}
    pathlib.Path(a.output).write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(report,indent=2))
    if failures:raise SystemExit(1)
if __name__=="__main__":main()
