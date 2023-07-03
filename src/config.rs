use std::{collections::HashMap, env, path::PathBuf};

use serde::{Deserialize, Serialize};
fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ProcTmuxConfig {
    procs: HashMap<String, ProcessConfig>,
}

fn default_kill_signal() -> String {
    "SIGKILL".to_string()
}
fn current_working_dir() -> String {
    get_current_working_dir().unwrap().to_str().unwrap().to_string()
}
fn default_autostart() -> bool {
    false 
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ProcessConfig{
    #[serde(default = "default_autostart")]
    autostart: bool,
    shell: Option<String>,
    cmd: Option<Vec<String>>,
    #[serde(default = "current_working_dir")]
    cwd: String,
    #[serde(default = "default_kill_signal")]
    stop: String,
    env: Option<HashMap<String, Option<String>>>,
    add_path: Option<Vec<String>>,
    description: Option<String>,
    docs: Option<String>,
    categories: Option<Vec<String>>,
    meta_tags: Option<Vec<String>>
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    #[test]
    fn deserializing_a_proctmux_config_works() {
        let proctmux_config_file = fs::File::open("./proctmux.yaml").unwrap();
        let proctmux_config: ProcTmuxConfig = serde_yaml::from_reader(proctmux_config_file).unwrap();
        assert!(!proctmux_config.procs.is_empty());
        let proc = proctmux_config.procs.get("tail log");
        assert!(proc.is_some());
        let proc = proc.unwrap();
        assert!(proc.autostart);
        assert_eq!(proc.cwd, get_current_working_dir().unwrap().to_str().unwrap());
        assert_eq!(proc.shell, Some("tail -f /tmp/term.log".to_string()));
    }
}
