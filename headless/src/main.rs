#[path = "../../src-tauri/src/policy.rs"]
mod policy;
use policy::classify_operation;
use serde::Serialize;
use std::{env, fs, path::{Path, PathBuf}, process::{Command, ExitStatus}};

const CLOUDOS_ROOT: &str = "/home/info/kmj-cloudos";

#[derive(Serialize)]
struct Probe<'a> { app:&'a str, mode:&'a str, policy_mode:&'a str, platform:&'a str, architecture:&'a str }

#[derive(Clone,Copy)]
enum Op { Inspect, GitStatus, DiffCheck, PyCompile, ProviderTests }
impl Op {
 fn parse(s:&str)->Option<Self>{match s{"inspect"=>Some(Self::Inspect),"git-status"=>Some(Self::GitStatus),"diff-check"=>Some(Self::DiffCheck),"py-compile"=>Some(Self::PyCompile),"provider-tests"=>Some(Self::ProviderTests),_=>None}}
 fn policy(self)->&'static str{match self{Self::Inspect=>"project.inspect",Self::GitStatus|Self::DiffCheck=>"git.status",_=>"test.run"}}
}
fn root(s:&str)->Result<PathBuf,String>{
 if s!=CLOUDOS_ROOT{return Err("CloudOS root denied".into())}
 let p=Path::new(s); if !p.is_absolute()||!p.join(".git").exists(){return Err("invalid CloudOS workspace".into())}
 let c=fs::canonicalize(p).map_err(|e|e.to_string())?;
 if c!=Path::new(CLOUDOS_ROOT){return Err("canonical root mismatch".into())} Ok(c)
}
fn run(r:&Path,p:&str,a:&[&str])->Result<ExitStatus,String>{Command::new(p).args(a).current_dir(r).status().map_err(|e|e.to_string())}
fn execute(r:&str,o:Op)->Result<ExitStatus,String>{
 let r=root(r)?; if !classify_operation(o.policy()).is_allowed(){return Err("policy denied".into())}
 match o{
  Op::Inspect|Op::GitStatus=>run(&r,"git",&["status","--short","--branch"]),
  Op::DiffCheck=>run(&r,"git",&["diff","--check"]),
  Op::PyCompile=>run(&r,"python3",&["-m","py_compile","src/hypervisor/runtime/libvirt_provider.py"]),
  Op::ProviderTests=>run(&r,"python3",&["-m","pytest","-q","tests/test_libvirt_provider.py"])
 }
}
fn main(){
 let a:Vec<String>=env::args().collect();
 match a.get(1).map(String::as_str).unwrap_or("probe"){
  "probe"=>println!("{}",serde_json::to_string_pretty(&Probe{app:"KMJ Desktop Commander",mode:"headless",policy_mode:"deny-by-default",platform:env::consts::OS,architecture:env::consts::ARCH}).unwrap()),
  "policy"=>{let Some(x)=a.get(2) else{std::process::exit(2)};println!("{}",serde_json::to_string_pretty(&classify_operation(x)).unwrap())},
  "cloudos"=>{let Some(n)=a.get(2) else{std::process::exit(2)};let Some(o)=Op::parse(n) else{eprintln!("denied by default");std::process::exit(3)};match execute(a.get(3).map(String::as_str).unwrap_or(CLOUDOS_ROOT),o){Ok(s)=>std::process::exit(s.code().unwrap_or(1)),Err(e)=>{eprintln!("{e}");std::process::exit(3)}}},
  _=>{eprintln!("denied by default");std::process::exit(3)}
 }
}
#[cfg(test)]
mod tests{use super::*;#[test]fn unknown_denied(){assert!(Op::parse("shell").is_none());assert!(Op::parse("system-reboot").is_none())}#[test]fn wrong_root_denied(){assert!(root("/tmp/kmj-cloudos").is_err());assert!(root("/home/info/kmj-cloudos;id").is_err())}#[test]fn allowlist_policy_allowed(){for o in[Op::Inspect,Op::GitStatus,Op::DiffCheck,Op::PyCompile,Op::ProviderTests]{assert!(classify_operation(o.policy()).is_allowed())}}}
