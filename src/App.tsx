import { type FormEvent, useEffect, useState } from "react";
import {
  commander,
  type JobRecord,
  type PolicyDecision,
  type RemoteProbeResult,
  type SystemProbe,
} from "./lib/commander";

export function App() {
  const [probe, setProbe] = useState<SystemProbe | null>(null);
  const [policy, setPolicy] = useState<PolicyDecision | null>(null);
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [host, setHost] = useState("");
  const [username, setUsername] = useState("");
  const [port, setPort] = useState(22);
  const [remote, setRemote] = useState<RemoteProbeResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  const refreshJobs = () => commander.jobs().then(setJobs).catch(() => undefined);

  useEffect(() => {
    commander.probe().then(setProbe).catch(() => undefined);
    commander.evaluate("remote.probe").then(setPolicy).catch(() => undefined);
    refreshJobs();
  }, []);

  async function connect(event: FormEvent) {
    event.preventDefault();
    setRunning(true);
    setError(null);
    setRemote(null);
    try {
      setRemote(await commander.remoteProbe({ host, username, port }));
    } catch (reason) {
      setError(String(reason));
    } finally {
      setRunning(false);
      refreshJobs();
    }
  }

  return (
    <main className="shell">
      <header className="topbar">
        <div><span className="eyebrow">KMJ TECHNO</span><h1>Desktop Commander</h1></div>
        <span className="status"><i /> POLICY ENFORCED</span>
      </header>

      <section className="hero compactHero">
        <div>
          <span className="eyebrow">ENGINEERING CONTROL PLANE</span>
          <h2>Operate safely.<br />Move fast.</h2>
          <p>Remote engineering operations cross a native Rust policy boundary before execution.</p>
        </div>
        <div className="health">
          <span>COMMANDER CORE</span>
          <strong>{probe ? "ONLINE" : "INITIALIZING"}</strong>
          <small>{probe ? `${probe.platform} · ${probe.architecture} · v${probe.app_version}` : "Native probe pending"}</small>
        </div>
      </section>

      <section className="grid">
        <article className="panel command">
          <div className="panelTitle"><span>REMOTE PROBE</span><b>01</b></div>
          <form className="remoteForm" onSubmit={connect}>
            <label>HOST<input value={host} onChange={(event) => setHost(event.target.value)} placeholder="server.example.com" required /></label>
            <label>USER<input value={username} onChange={(event) => setUsername(event.target.value)} placeholder="deploy" required /></label>
            <label>PORT<input type="number" min="1" max="65535" value={port} onChange={(event) => setPort(Number(event.target.value))} required /></label>
            <button type="submit" disabled={running || !policy?.allowed}>{running ? "PROBING…" : "RUN SAFE PROBE"}</button>
          </form>
          <p className="hint">Uses your OS OpenSSH configuration/agent. Host keys must already be trusted. Password prompts are disabled.</p>
          {error && <pre className="output errorOutput">{error}</pre>}
          {remote && <pre className={`output ${remote.success ? "okOutput" : "errorOutput"}`}>{remote.output || `Exit ${remote.exit_code ?? "unknown"}`}</pre>}
        </article>

        <article className="panel">
          <div className="panelTitle"><span>PERSISTENT JOBS</span><b>02</b></div>
          <div className="jobs">
            {jobs.length === 0 && <p className="empty">No operations recorded yet.</p>}
            {jobs.slice(0, 6).map((job) => (
              <div className="job" key={job.id}>
                <div><strong>{job.operation}</strong><span>{job.target}</span></div>
                <b className={`jobStatus ${job.status}`}>{job.status}</b>
              </div>
            ))}
          </div>
        </article>

        <article className="panel wide">
          <div className="panelTitle"><span>TRUST BOUNDARY</span><b>03</b></div>
          <div className="trust">
            <div><strong>UI</strong><span>Structured intent</span></div><em>→</em>
            <div><strong>RUST POLICY</strong><span>Deny by default</span></div><em>→</em>
            <div><strong>OPENSSH</strong><span>Fixed operation</span></div><em>→</em>
            <div><strong>JOB LEDGER</strong><span>Persistent result</span></div>
          </div>
        </article>
      </section>
    </main>
  );
}
