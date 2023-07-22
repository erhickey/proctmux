use std::{collections::HashMap, env, path::PathBuf};

use serde::{Deserialize, Serialize, Deserializer};

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}


fn default_general() -> GeneralConfig {
    GeneralConfig {
        detached_session_name: default_detached_session_name(),
        kill_existing_session: default_kill_existing_session(),
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ProcTmuxConfig {

    #[serde(default = "default_general")]
    pub general: GeneralConfig,
    pub procs: HashMap<String, ProcessConfig>,
    pub keybinding: KeybindingConfig,
    pub log_file: String,
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
fn default_quit_keybinding() -> Vec<String> {
    vec!["q".to_string()]
}
fn default_start_keybinding() -> Vec<String> {
    vec!["s".to_string()]
}
fn default_stop_keybinding() -> Vec<String> {
    vec!["x".to_string()]
}
fn default_up_keybinding() -> Vec<String> {
    vec!["up".to_string(), "k".to_string()]
}
fn default_down_keybinding() -> Vec<String> {
    vec!["down".to_string(), "j".to_string()]
}
fn default_filter_keybinding() -> Vec<String> {
    vec!["/".to_string()]
}
fn default_filter_submit_keybinding() -> Vec<String> {
    vec!["\n".to_string()]
}
fn deserialize_keybinding_notation<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // map any textual representations of keybinding 
    // characters into the stdin characters that need to be detected 
    let key_codes: Vec<String> = Deserialize::deserialize(deserializer)?;
    let new_codes = key_codes.iter().map(|key| {
        if key.to_lowercase().eq("enter") {
            return "\n".to_string() 
        }
        key.to_string()
    }).collect();
    Ok(new_codes)
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct KeybindingConfig {
    // quit: List[str] = field(default_factory=lambda: ['q'])
    // filter: List[str] = field(default_factory=lambda: ['/'])
    // submit_filter: List[str] = field(default_factory=lambda: ['enter'])
    // next_input: List[str] = field(default_factory=lambda: ['tab', 'down'])
    // previous_input: List[str] = field(default_factory=lambda: ['s-tab', 'up'])
    // submit_dialog: List[str] = field(default_factory=lambda: ['enter'])
    // cancel_dialog: List[str] = field(default_factory=lambda: ['escape'])
    // start: List[str] = field(default_factory=lambda: ['s'])
    // stop: List[str] = field(default_factory=lambda: ['x'])
    // up: List[str] = field(default_factory=lambda: ['up', 'k'])
    // down: List[str] = field(default_factory=lambda: ['down', 'j'])
    // switch_focus: List[str] = field(default_factory=lambda: ['c-w'])
    // zoom: List[str] = field(default_factory=lambda: ['c-z'])
    // docs: List[str] = field(default_factory=lambda: ['?'])
    // toggle_scroll: List[str] = field(default_factory=lambda: ['c-s'])
    // #[serde(default = "default_quit_keybinding")]
    // quit: Vec<String>,
    // filter: Option<Vec<String>>,
    // submit_filter: Option<Vec<String>>,
    // next_input: Option<Vec<String>>,
    // previous_input: Option<Vec<String>>,
    // submit_dialog: Option<Vec<String>>,
    // cancel_dialog: Option<Vec<String>>,
    // switch_focus: Option<Vec<String>>,
    // zoom: Option<Vec<String>>,
    // docs: Option<Vec<String>>,

    #[serde(default = "default_quit_keybinding", deserialize_with = "deserialize_keybinding_notation")]
    pub quit: Vec<String>,
    #[serde(default = "default_start_keybinding", deserialize_with = "deserialize_keybinding_notation")]
    pub start: Vec<String>,
    #[serde(default = "default_stop_keybinding",deserialize_with = "deserialize_keybinding_notation")]
    pub stop: Vec<String>,
    #[serde(default = "default_up_keybinding",deserialize_with = "deserialize_keybinding_notation")]
    pub up: Vec<String>,
    #[serde(default = "default_down_keybinding" ,deserialize_with = "deserialize_keybinding_notation")]
    pub down: Vec<String>,
    #[serde(default = "default_filter_keybinding", deserialize_with = "deserialize_keybinding_notation")]
    pub filter: Vec<String>,
    #[serde(default = "default_filter_submit_keybinding", deserialize_with = "deserialize_keybinding_notation")]
    pub filter_submit: Vec<String>,
}


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessConfig{
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    pub shell: Option<String>,
    pub cmd: Option<Vec<String>>,
    #[serde(default = "current_working_dir")]
    pub cwd: String,
    #[serde(default = "default_kill_signal")]
    pub stop: String,
    pub env: Option<HashMap<String, Option<String>>>,
    pub add_path: Option<Vec<String>>,
    pub description: Option<String>,
    pub docs: Option<String>,
    pub categories: Option<Vec<String>>,
    pub meta_tags: Option<Vec<String>>
}

fn default_detached_session_name() -> String {
    "proctmux".to_string()
}

fn default_kill_existing_session() -> bool {
    false
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GeneralConfig{
    #[serde(default = "default_detached_session_name")]
    pub detached_session_name: String,
    #[serde(default = "default_kill_existing_session")]
    pub kill_existing_session: bool,
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
