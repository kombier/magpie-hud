//! Read title metadata through Magpie's product CLI, never an Agent's internal store.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct SessionTitle {
    agent: String,
    id: String,
    #[serde(default)]
    title: String,
}

pub fn refresh(titles: &mut HashMap<String, String>) -> Result<(), String> {
    let home = crate::config::directories_base();
    let executable = [home.join(".local/bin/magpie"), PathBuf::from("/opt/homebrew/bin/magpie"), PathBuf::from("/usr/local/bin/magpie")]
        .into_iter().find(|path| path.is_file()).unwrap_or_else(|| PathBuf::from("magpie"));
    let mut command = Command::new(executable);
    command.args(["sessions", "--json"]).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW for the metadata reader.
    }
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() { return Err("Magpie sessions command failed".into()); }
    let rows: Vec<SessionTitle> = serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    merge(titles, rows);
    Ok(())
}

fn merge(titles: &mut HashMap<String, String>, rows: Vec<SessionTitle>) {
    for row in rows.into_iter().filter(|row| row.agent.eq_ignore_ascii_case("zcode")) {
        let title: String = row.title.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(160).collect();
        // ZCode stores sess_<uuid>, while its request header carries the same UUID alone.
        for id in [Some(row.id.as_str()), row.id.strip_prefix("sess_")].into_iter().flatten() {
            titles.remove(id);
            if !title.is_empty() { titles.insert(id.to_string(), title.clone()); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zcode_header_identity_and_native_title_stay_paired_across_renames() {
        let mut titles = HashMap::new();
        let rows = |title| vec![SessionTitle {agent: "zcode".into(), id: "sess_chat-a".into(), title},
                               SessionTitle {agent: "codex".into(), id: "chat-b".into(), title: "unrelated".into()}];
        merge(&mut titles, rows("Original title".into()));
        assert_eq!(titles.get("chat-a").map(String::as_str), Some("Original title"));
        assert_eq!(titles.get("sess_chat-a"), titles.get("chat-a"));
        assert!(!titles.contains_key("chat-b"));
        merge(&mut titles, rows(" Renamed\nconversation ".into()));
        assert_eq!(titles.get("chat-a").map(String::as_str), Some("Renamed conversation"));
        merge(&mut titles, rows(String::new()));
        assert!(titles.is_empty());
    }
}
