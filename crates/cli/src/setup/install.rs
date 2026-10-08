//! Installing the node software: Bitcoin Core, from Homebrew.
//!
//! Homebrew builds and checks Bitcoin Core itself and keeps it updated with
//! the rest of the machine, so this program hands the job to it rather than
//! downloading binaries of its own.

use std::process::{Command, Stdio};
use std::time::Instant;

use super::screen::{Answer, Part, Wizard};
use crate::config::Settings;
use crate::dashboard::format;
use crate::{node, platform};

/// Offers to install Bitcoin Core, or to update the one found (`found`).
///
/// On success, the node software at `settings.binaries` is ready to start.
pub fn offer(
    wizard: &mut Wizard,
    settings: &mut Settings,
    found: Option<(u32, u32, u32)>,
) -> Result<Answer<()>, String> {
    let intro = match found {
        None => vec![Part::Text(
            "The node is the program that talks to the Bitcoin network and checks every block for \
             itself. It's called Bitcoin Core, and it's free."
                .into(),
        )],
        Some(version) => vec![Part::Text(format!(
            "This computer has Bitcoin Core {}, and this program needs {} or newer.",
            node::version_string(version),
            node::version_string(node::MIN_VERSION)
        ))],
    };
    let verb = if found.is_some() { "upgrade" } else { "install" };

    if !has_homebrew() {
        let mut how = intro;
        how.extend([
            Part::Gap,
            Part::Text(
                "The easiest way is Homebrew: install it from brew.sh, then run this again and it \
                 will do the rest. Or download Bitcoin Core from bitcoincore.org yourself and make \
                 sure bitcoind is on your PATH."
                    .into(),
            ),
        ]);
        wizard.notice(&how, "enter quit")?;
        return Ok(Answer::Quit);
    }

    let choices = vec![
        format!("{} Bitcoin Core with Homebrew   (brew {verb} bitcoin)", capitalised(verb)),
        "I'll do it myself — show me how".to_owned(),
    ];
    match wizard.choose(&intro, &choices)? {
        Answer::Given(0) => brew(wizard, settings, verb),
        Answer::Given(_) => {
            let how = [
                Part::Text(format!("In a terminal, run:  brew {verb} bitcoin")),
                Part::Gap,
                Part::Text("Then run btc-miner again and it carries on from here.".into()),
            ];
            wizard.notice(&how, "enter quit")?;
            Ok(Answer::Quit)
        }
        Answer::Back | Answer::Quit => Ok(Answer::Quit),
    }
}

/// `install` → `Install`.
fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
}

fn has_homebrew() -> bool {
    Command::new("brew")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Runs `brew <verb> bitcoin`, showing its latest line of output as it goes.
fn brew(wizard: &mut Wizard, settings: &mut Settings, verb: &str) -> Result<Answer<()>, String> {
    let log = platform::state_dir().join("brew.log");
    std::fs::create_dir_all(platform::state_dir()).map_err(|error| error.to_string())?;
    let output =
        std::fs::File::create(&log).map_err(|error| format!("cannot write {}: {error}", log.display()))?;
    let errors = output.try_clone().map_err(|error| error.to_string())?;
    let mut child = Command::new("brew")
        .args([verb, "bitcoin"])
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(errors)
        .spawn()
        .map_err(|error| format!("cannot run Homebrew: {error}"))?;

    let started = Instant::now();
    let mut status = None;
    let finished = wizard.watch("q quit (Homebrew carries on by itself)", || {
        if let Some(done) = child.try_wait().map_err(|error| error.to_string())? {
            status = Some(done);
            return Ok(None);
        }
        Ok(Some(vec![
            Part::Text(format!(
                "Homebrew is {verb}ing Bitcoin Core… {} so far.",
                format::duration(started.elapsed().as_secs_f64())
            )),
            Part::Gap,
            Part::Text(last_line(&log)),
        ]))
    })?;
    if !finished {
        return Ok(Answer::Quit);
    }
    if !status.is_some_and(|status| status.success()) {
        return Err(format!(
            "Homebrew could not {verb} Bitcoin Core: {} (the full output is in {})",
            last_line(&log),
            log.display()
        ));
    }

    settings.binaries = platform::default_node_binaries();
    let version = node::installed_version(settings)?;
    if version < node::MIN_VERSION {
        return Err(format!(
            "Homebrew installed Bitcoin Core {}, still too old",
            node::version_string(version)
        ));
    }
    Ok(Answer::Given(()))
}

/// The last non-empty line of a file, or "".
fn last_line(path: &std::path::Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        assert_eq!(capitalised("install"), "Install");
        assert_eq!(capitalised(""), "");
    }
}
