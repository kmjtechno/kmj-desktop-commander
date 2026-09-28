import { useEffect, useState } from "react";
import { commander, type PolicyDecision, type SystemProbe } from "./lib/commander";

const gates = [
  ["Native Core", "Ready"],
  ["Policy Engine", "Ready"],
  ["TypeScript", "Ready"],
  ["Remote Runner", "Next slice"],
  ["Git Workflow", "Next slice"],
] as const;

export function App() {
  const [probe, setProbe] = useState<SystemProbe | null>(null);
  const [policy, setPolicy] = useState<PolicyDecision | null>(null);

  useEffect(() => {
    commander.probe().then(setProbe).catch(() => undefined);
    commander.evaluate("project.inspect").then(setPolicy).catch(() => undefined);
  }, []);

  return <main className="shell">
    <header className="topbar">
      <div><span className="eyebrow">KMJ TECHNO</span><h1>Desktop Commander</h1></div>
      <span className="status"><i /> SECURITY-FIRST</span>
    </header>
    <section className="hero">
      <div>
        <span className="eyebrow">ENGINEERING CONTROL PLANE</span>
        <h2>One command surface.<br />Every engineering system.</h2>
        <p>Native operations, remote runners, quality gates, Git workflows and AI orchestration behind a policy-controlled Rust boundary.</p>
      </div>
      <div className="health"><span>COMMANDER CORE</span><strong>{probe ? "ONLINE" : "INITIALIZING"}</strong><small>{probe ? `${probe.platform} · ${probe.architecture}` : "Native probe pending"}</small></div>
    </section>
    <section className="grid">
      <article className="panel">
        <div className="panelTitle"><span>PROJECT HEALTH</span><b>01</b></div>
        {gates.map(([name,value]) => <div className="gate" key={name}><span>{name}</span><strong>{value}</strong></div>)}
      </article>
      <article className="panel command">
        <div className="panelTitle"><span>KRISTI COMMAND</span><b>02</b></div>
        <div className="prompt"><span>›</span><p>Inspect the active project, resolve safe blockers, run quality gates, and prepare a verified diff.</p></div>
        <div className="policy"><span>POLICY</span><strong>{policy?.allowed ? "SAFE INSPECTION ALLOWED" : "DENY BY DEFAULT"}</strong></div>
        <button type="button" disabled>REMOTE RUNNER · NEXT SLICE</button>
      </article>
      <article className="panel wide">
        <div className="panelTitle"><span>TRUST BOUNDARY</span><b>03</b></div>
        <div className="trust"><div><strong>UI</strong><span>Intent only</span></div><em>→</em><div><strong>RUST POLICY</strong><span>Classify + authorize</span></div><em>→</em><div><strong>RUNNER</strong><span>Scoped execution</span></div><em>→</em><div><strong>AUDIT</strong><span>Verified result</span></div></div>
      </article>
    </section>
  </main>;
}
