use std::collections::HashSet;

use anyhow::Result;
use sysinfo::{Pid, System};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Process {
    pub pid: Pid,
}

use anyhow::anyhow;

impl Process {
    pub fn current() -> Self {
        Self {
            pid: sysinfo::get_current_pid().expect("Failed to get current process ID"),
        }
    }

    pub fn find(pid: Pid) -> Option<Self> {
        System::new_all().process(pid).map(|_| Self { pid })
    }

    pub fn children(&self) -> Result<HashSet<Self>> {
        let mut procs: HashSet<Self> = HashSet::new();
        let sys = System::new_all();

        for (pid, proc) in sys.processes().iter() {
            if let Some(parent_pid) = proc.parent() {
                if parent_pid == self.pid {
                    let child_proc = Self { pid: *pid };
                    procs.insert(child_proc.clone());
                    // Handle the result from the recursive call
                    match child_proc.children() {
                        Ok(grandchildren) => procs.extend(grandchildren),
                        Err(e) => {
                            log::error!(
                                "Failed to get children of process {}: {}",
                                child_proc.pid,
                                e
                            );
                            // Decide if you want to continue or return the error
                            // For now, we'll log and continue, which is safer than panicking
                        }
                    }
                }
            }
        }

        Ok(procs)
    }

    pub fn kill(&self) -> Result<bool> {
        let sys = System::new_all();

        self.children()?
            .iter()
            .all(|proc| match sys.process(proc.pid) {
                Some(p) => p.kill(),
                None => false,
            });

        if let Some(parent) = sys.process(self.pid) {
            Ok(parent.kill())
        } else {
            Ok(false)
        }
    }
}
