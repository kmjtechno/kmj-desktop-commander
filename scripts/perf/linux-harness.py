#!/usr/bin/env python3
import argparse,json,math,os,pathlib,re,shutil,signal,subprocess,tempfile,threading,time
READY="KMJ_PERF_UI_READY"
def pct(v,p):
    if not v: raise RuntimeError("no samples")
    v=sorted(v); return v[max(0,math.ceil(len(v)*p)-1)]
def descendants(root):
    wanted={root}
    while True:
        before=len(wanted)
        for proc in pathlib.Path("/proc").iterdir():
            if not proc.name.isdigit(): continue
            try:
                s=(proc/"stat").read_text().split(); pid=int(proc.name); ppid=int(s[3])
                if ppid in wanted: wanted.add(pid)
            except Exception: pass
        if len(wanted)==before: return wanted
def find_executable_descendant(root,binary,timeout=3):
    target=str(binary.resolve());end=time.monotonic()+timeout
    while time.monotonic()<end:
        for pid in descendants(root)-{root}:
            try:
                if os.path.realpath(f"/proc/{pid}/exe")==target:return pid
            except Exception:pass
        time.sleep(.02)
    raise RuntimeError("Commander child process not found beneath trace wrapper")
def sample(root):
    ticks=0; rss=0
    for pid in descendants(root):
        try:
            s=pathlib.Path(f"/proc/{pid}/stat").read_text().split(); ticks+=int(s[13])+int(s[14])
            for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines():
                if line.startswith("VmRSS:"): rss+=int(line.split()[1]); break
        except Exception: pass
    return ticks,rss/1024
def stop(p):
    if p.poll() is not None:return
    try: os.killpg(p.pid,signal.SIGTERM)
    except ProcessLookupError:return
    try:p.wait(timeout=3)
    except subprocess.TimeoutExpired:
        try:os.killpg(p.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        p.wait(timeout=3)
def env(root):
    e=os.environ.copy();e.update({"KMJ_PERF_HARNESS":"1","KMJ_PERF_READY_FILE":str(root/"ui-ready"),"XDG_DATA_HOME":str(root/"data"),"XDG_CONFIG_HOME":str(root/"config"),"XDG_CACHE_HOME":str(root/"cache")});return e
def launch(binary,state,cmd=None):
    (state/"ui-ready").unlink(missing_ok=True)
    return subprocess.Popen(cmd or [str(binary)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env(state),start_new_session=True,bufsize=1)
def ready(p,state,timeout=15):
    start=time.perf_counter();event=threading.Event();lines=[]
    def read():
        if p.stdout:
            for line in p.stdout:
                lines.append(line)
                if READY in line:event.set()
    threading.Thread(target=read,daemon=True).start()
    deadline=time.monotonic()+timeout
    ready_file=state/"ui-ready"
    while time.monotonic()<deadline:
        if event.is_set() or ready_file.exists():
            return (time.perf_counter()-start)*1000
        if p.poll() is not None:
            break
        time.sleep(.01)
    stop(p);raise RuntimeError("UI readiness marker not observed: "+"".join(lines[-20:]))
def startups(binary,base,count,cold):
    vals=[];state=base/("cold" if cold else "warm");state.mkdir(parents=True,exist_ok=True)
    if not cold:
        p=launch(binary,state)
        try:ready(p,state)
        finally:stop(p)
    for _ in range(count):
        if cold:shutil.rmtree(state,ignore_errors=True);state.mkdir(parents=True)
        p=launch(binary,state)
        try:vals.append(ready(p,state))
        finally:stop(p)
    return vals
def idle(binary,base,warmup,duration,interval):
    trace=base/"network.strace";state=base/"idle";state.mkdir(parents=True,exist_ok=True)
    p=launch(binary,state,["strace","-f","-ttt","-e","trace=network","-o",str(trace),str(binary)])
    try:
        ready(p,state);app_pid=find_executable_descendant(p.pid,binary);time.sleep(warmup);idle_start=time.time();hz=os.sysconf(os.sysconf_names["SC_CLK_TCK"])
        cpus=[];rss=[];prev_t,_=sample(app_pid);prev=time.monotonic();end=prev+duration
        while time.monotonic()<end:
            time.sleep(interval);now=time.monotonic();ticks,mem=sample(app_pid);elapsed=max(now-prev,.001)
            cpus.append(max(0,(ticks-prev_t)/hz/elapsed*100));rss.append(mem);prev_t=ticks;prev=now
    finally:stop(p)
    outbound=0
    for line in trace.read_text(errors="replace").splitlines():
        m=re.match(r"^(?:(?:\[pid\s+\d+\]|\d+)\s+)?(\d+\.\d+)\s+.*connect\(",line)
        if m and float(m.group(1))>=idle_start and ("AF_INET" in line or "AF_INET6" in line):outbound+=1
    return cpus,rss,outbound,trace
def mib(p):return p.stat().st_size/1048576
def main():
    ap=argparse.ArgumentParser();ap.add_argument("--binary",required=True);ap.add_argument("--bundle-dir",required=True);ap.add_argument("--output-dir",required=True);ap.add_argument("--config",default="performance-budgets.json");a=ap.parse_args()
    cfg=json.load(open(a.config));mc=cfg["measurement"];out=pathlib.Path(a.output_dir);out.mkdir(parents=True,exist_ok=True);binary=pathlib.Path(a.binary).resolve();state=pathlib.Path(tempfile.mkdtemp(prefix="kmj-perf-"))
    try:
        n=int(mc["startup_samples"]);cold=startups(binary,state,n,True);warm=startups(binary,state,n,False)
        cpu,rss,net,trace=idle(binary,state,int(mc["warmup_seconds"]),int(mc["idle_window_seconds"]),int(mc["cpu_sample_interval_ms"])/1000)
        bundles=[p for p in pathlib.Path(a.bundle_dir).rglob("*") if p.is_file() and p.suffix.lower() in {".appimage",".deb",".exe",".msi",".dmg"}]
        if not bundles:raise RuntimeError("no installer/package found")
        metrics={"schema":1,"idle_cpu_p95_percent":round(pct(cpu,.95),4),"idle_cpu_max_percent":round(max(cpu),4),"idle_rss_p95_mib":round(pct(rss,.95),4),"idle_rss_max_mib":round(max(rss),4),"idle_network_requests_10m":net,"startup_warm_p95_ms":round(pct(warm,.95),2),"startup_cold_p95_ms":round(pct(cold,.95),2),"startup_max_ms":round(max(warm+cold),2),"stripped_executable_mib":round(mib(binary),4),"installer_mib":round(max(mib(p) for p in bundles),4)}
        (out/"metrics.json").write_text(json.dumps(metrics,indent=2)+"\n");(out/"samples.json").write_text(json.dumps({"warm_startup_ms":warm,"cold_startup_ms":cold,"cpu_percent":cpu,"rss_mib":rss},indent=2)+"\n");shutil.copy2(trace,out/"network.strace");print(json.dumps(metrics,indent=2))
    finally:shutil.rmtree(state,ignore_errors=True)
if __name__=="__main__":main()
