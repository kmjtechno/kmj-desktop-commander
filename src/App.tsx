import { type FormEvent, useEffect, useState } from "react";
import { fetchCommanderBootstrap, type CommanderBootstrap } from "./lib/commercial";
import {
  commander,
  type JobRecord,
  type RemoteOperation,
  type RemoteResult,
  type SavedProfile,
  type SystemProbe,
} from "./lib/commander";

const operations: Array<[RemoteOperation, string, string]> = [
  ["probe", "Probe Server", "Read-only host and OS check"],
  ["project_inspect", "Inspect Project", "Branch, status and detected stack"],
  ["git_status", "Git Status", "Current branch and changed files"],
  ["git_diff_check", "Diff Check", "Whitespace errors and diff summary"],
  ["php_test", "PHP Tests", "Run php artisan test"],
  ["frontend_typecheck", "TypeScript", "Run pnpm typecheck"],
  ["frontend_build", "Frontend Build", "Run pnpm build"],
  ["rust_test", "Rust Tests", "Run cargo test"],
];

const blank = (): SavedProfile => ({
  id: "",
  label: "",
  host: "",
  username: "",
  port: 22,
  project_root: "",
});

export function App() {
  const [system, setSystem] = useState<SystemProbe | null>(null);
  const [commercial, setCommercial] = useState<CommanderBootstrap | null>(null);
  const [profiles, setProfiles] = useState<SavedProfile[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [draft, setDraft] = useState<SavedProfile>(blank());
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [result, setResult] = useState<RemoteResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState<RemoteOperation | null>(null);

  const refreshJobs = () => commander.jobs().then(setJobs).catch(() => undefined);

  useEffect(() => {
    commander.probe().then(setSystem).catch(() => undefined);
    fetchCommanderBootstrap().then(setCommercial).catch(() => undefined);
    commander.profiles().then((items) => {
      setProfiles(items);
      if (items[0]) {
        setSelectedId(items[0].id);
        setDraft(items[0]);
      }
    }).catch(() => undefined);
    refreshJobs();
  }, []);

  function selectProfile(id: string) {
    const profile = profiles.find((item) => item.id === id);
    if (!profile) return;
    setSelectedId(id);
    setDraft(profile);
    setResult(null);
    setError(null);
  }

  async function save(event: FormEvent) {
    event.preventDefault();
    setError(null);
    const profile = {
      ...draft,
      id: draft.id || `profile-${Date.now()}`,
      label: draft.label.trim(),
      host: draft.host.trim(),
      username: draft.username.trim(),
      project_root: draft.project_root.trim(),
    };
    try {
      const items = await commander.saveProfile(profile);
      setProfiles(items);
      setSelectedId(profile.id);
      setDraft(profile);
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function remove() {
    if (!draft.id) return;
    try {
      const items = await commander.deleteProfile(draft.id);
      setProfiles(items);
      const next = items[0] ?? blank();
      setSelectedId(next.id);
      setDraft(next);
      setResult(null);
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function run(operation: RemoteOperation) {
    setRunning(operation);
    setError(null);
    setResult(null);
    try {
      const remote = await commander.execute({
        host: draft.host,
        username: draft.username,
        port: draft.port,
        project_root: operation === "probe" ? null : draft.project_root,
      }, operation);
      setResult(remote);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setRunning(null);
      refreshJobs();
    }
  }

  return (
    <main className="shell">
      <header className="topbar">
        <div><span className="eyebrow">KMJ TECHNO</span><h1>Desktop Commander</h1></div>
        <span className="status"><i /> {system ? `CORE ONLINE · v${system.app_version}` : "STARTING"}{commercial ? ` · ${commercial.channel.toUpperCase()} · ${commercial.default_plan.toUpperCase()}` : ""}</span>
      </header>

      <section className="workspace">
        <aside className="sidebar">
          <div className="panelTitle"><span>SERVER PROFILES</span><b>{profiles.length}</b></div>
          <div className="profileList">
            {profiles.map((profile) => (
              <button className={selectedId === profile.id ? "profile active" : "profile"} key={profile.id} onClick={() => selectProfile(profile.id)}>
                <strong>{profile.label}</strong><span>{profile.username}@{profile.host}</span>
              </button>
            ))}
          </div>
          <button className="newProfile" onClick={() => { setSelectedId(""); setDraft(blank()); }}>+ NEW PROFILE</button>
        </aside>

        <section className="content">
          <div className="intro">
            <div><span className="eyebrow">SAFE REMOTE OPERATIONS</span><h2>{draft.label || "Configure a server"}</h2></div>
            <span className="trustBadge">DENY BY DEFAULT</span>
          </div>

          <form className="profileForm" onSubmit={save}>
            <label>PROFILE NAME<input value={draft.label} onChange={(e) => setDraft({...draft, label:e.target.value})} placeholder="KMJ Platform" required /></label>
            <label>HOST<input value={draft.host} onChange={(e) => setDraft({...draft, host:e.target.value})} placeholder="server.example.com" required /></label>
            <label>USER<input value={draft.username} onChange={(e) => setDraft({...draft, username:e.target.value})} placeholder="deploy" required /></label>
            <label>PORT<input type="number" min="1" max="65535" value={draft.port} onChange={(e) => setDraft({...draft, port:Number(e.target.value)})} required /></label>
            <label className="rootField">PROJECT ROOT<input value={draft.project_root} onChange={(e) => setDraft({...draft, project_root:e.target.value})} placeholder="/srv/project" required /></label>
            <div className="profileActions"><button type="submit">SAVE PROFILE</button>{draft.id && <button type="button" className="danger" onClick={remove}>DELETE</button>}</div>
          </form>

          <p className="hint">Credentials are not stored. Commander uses your operating system OpenSSH agent/configuration with strict host-key verification and no password prompt.</p>

          <div className="operationGrid">
            {operations.map(([id, title, description]) => (
              <button className="operation" key={id} disabled={Boolean(running) || !draft.host || !draft.username || (id !== "probe" && !draft.project_root)} onClick={() => run(id)}>
                <span>{running === id ? "RUNNING" : "READY"}</span><strong>{title}</strong><small>{description}</small>
              </button>
            ))}
          </div>

          {(result || error) && <section className="console">
            <div className="panelTitle"><span>{result?.operation ?? "ERROR"}</span><b>{result ? (result.success ? "PASS" : "FAIL") : "BLOCKED"}</b></div>
            <pre>{error ?? result?.output ?? "No output"}</pre>
          </section>}

          <section className="history">
            <div className="panelTitle"><span>RECENT JOBS</span><b>{jobs.length}</b></div>
            {jobs.slice(0, 8).map((job) => <div className="job" key={job.id}>
              <div><strong>{job.operation}</strong><span>{job.target}</span></div>
              <b className={`jobStatus ${job.status}`}>{job.status}</b>
            </div>)}
            {jobs.length === 0 && <p className="empty">No operations recorded yet.</p>}
          </section>
        </section>
      </section>
    </main>
  );
}
