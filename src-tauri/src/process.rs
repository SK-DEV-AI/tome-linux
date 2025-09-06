use std::collections::HashSet;

use anyhow::{anyhow, Result};
use sysinfo::{Pid, System};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Process {
    pub pid: Pid,
}

impl Process {
    pub fn current() -> Result<Self> {
        sysinfo::get_current_pid()
            .map(|pid| Self { pid })
            .map_err(|e| anyhow!("Failed to get current process ID: {}", e))
    }

    pub fn children(&self, sys: &System) -> HashSet<Self> {
        let mut procs = HashSet::new();
        for (pid, proc) in sys.processes().iter() {
            if let Some(parent_pid) = proc.parent() {
                if parent_pid == self.pid {
                    let child_proc = Self { pid: *pid };
                    procs.insert(child_proc.clone());
                    procs.extend(child_proc.children(sys));
                }
            }
        }
        procs
    }

    pub fn kill_tree(&self) -> Result<bool> {
        let mut sys = System::new();
        sys.refresh_all();

        let children = self.children(&sys);
        for proc in children {
            if let Some(p) = sys.process(proc.pid) {
                p.kill();
            }
        }

        if let Some(parent) = sys.process(self.pid) {
            Ok(parent.kill())
        } else {
            Ok(false)
        }
    }
}
