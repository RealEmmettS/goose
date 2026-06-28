use clap::{Parser, Subcommand, ValueEnum};
use honk_engine::PokeAction;
use std::ffi::OsString;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "honk300",
    version,
    about = "A desktop goose for your screen",
    after_help = "Goose-speak:\n  <name> plz                 Start the goose\n  <name> bad | no | no honk  Stop the goose\n  <name> do honk             Poke a honk\n\nInstalled names planned for M19: honk300, honk, goose."
)]
pub struct Cli {
    /// Start the goose muted.
    #[arg(long, alias = "silent", global = true)]
    pub no_sound: bool,

    /// Disable cursor-stealing behavior for this run.
    #[arg(long, global = true)]
    pub no_mouse_steal: bool,

    /// Disable foreign-window ride behavior for this run.
    #[arg(long, global = true)]
    pub no_window_ride: bool,

    /// Use a specific config.toml instead of the per-user default.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Request native Wayland mode when the M18 backend exists.
    #[arg(long, global = true)]
    pub wayland: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Start the goose. This is also the default when no command is provided.
    Start,
    /// Stop the running goose through local IPC.
    Stop,
    /// Ask the running goose to reload runtime options.
    Reload,
    /// Open the terminal config editor.
    Config,
    /// Poke the running goose into a specific action.
    Do {
        #[arg(value_enum)]
        action: CliPokeAction,
    },
    /// Placeholder for the M19 installer flow.
    Install,
    /// Placeholder for the M19 uninstaller flow.
    Uninstall,
    /// Placeholder for the M19 updater flow.
    Update,
    /// Create or refresh the user config file.
    Setup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliPokeAction {
    Honk,
    Wander,
    Mud,
    Meme,
    Note,
    Nab,
}

impl CliPokeAction {
    pub fn into_engine(self) -> PokeAction {
        match self {
            Self::Honk => PokeAction::Honk,
            Self::Wander => PokeAction::Wander,
            Self::Mud => PokeAction::Mud,
            Self::Meme => PokeAction::Meme,
            Self::Note => PokeAction::Note,
            Self::Nab => PokeAction::Nab,
        }
    }
}

impl Cli {
    pub fn parse_normalized() -> Self {
        Self::parse_from(normalize_args(std::env::args_os()))
    }

    #[cfg(test)]
    fn try_parse_normalized<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        Self::try_parse_from(normalize_args(itr))
    }

    #[cfg(test)]
    pub fn is_start(&self) -> bool {
        self.command.is_none() || self.command == Some(Command::Start)
    }

    pub fn is_client_command(&self) -> bool {
        matches!(
            self.command,
            Some(Command::Stop | Command::Reload | Command::Do { .. })
        )
    }
}

fn normalize_args<I, T>(itr: I) -> Vec<String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut args: Vec<String> = itr
        .into_iter()
        .map(|arg| arg.into().to_string_lossy().into_owned())
        .collect();
    if args.is_empty() {
        args.push("honk300".into());
        return args;
    }

    let command_idx = first_command_index(&args);
    if command_idx >= args.len() {
        return args;
    }

    let token = args[command_idx].to_ascii_lowercase();
    match token.as_str() {
        "plz" => args[command_idx] = "start".into(),
        "bad" | "no" => {
            if token == "no"
                && args
                    .get(command_idx + 1)
                    .is_some_and(|next| next.eq_ignore_ascii_case("honk"))
            {
                args.remove(command_idx + 1);
            }
            args[command_idx] = "stop".into();
        }
        _ => {}
    }
    args
}

fn first_command_index(args: &[String]) -> usize {
    let mut idx = 1;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--config" {
            idx += 2;
        } else if arg.starts_with("--config=")
            || matches!(
                arg.as_str(),
                "--no-sound" | "--silent" | "--no-mouse-steal" | "--no-window-ride" | "--wayland"
            )
        {
            idx += 1;
        } else {
            break;
        }
    }
    idx
}

#[cfg(test)]
fn invoked_name(arg0: &str) -> String {
    Path::new(arg0)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("honk300")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command_defaults_to_start() {
        let cli = Cli::try_parse_normalized(["honk300"]).unwrap();
        assert!(cli.is_start());
    }

    #[test]
    fn parses_explicit_start() {
        let cli = Cli::try_parse_normalized(["honk300", "start"]).unwrap();
        assert!(cli.is_start());
    }

    #[test]
    fn parses_stop_reload_and_do() {
        let stop = Cli::try_parse_normalized(["honk300", "stop"]).unwrap();
        assert_eq!(stop.command, Some(Command::Stop));

        let reload = Cli::try_parse_normalized(["honk300", "reload"]).unwrap();
        assert_eq!(reload.command, Some(Command::Reload));

        let do_note = Cli::try_parse_normalized(["honk300", "do", "note"]).unwrap();
        assert_eq!(
            do_note.command,
            Some(Command::Do {
                action: CliPokeAction::Note
            })
        );
    }

    #[test]
    fn maps_cli_actions_to_engine_actions() {
        assert_eq!(CliPokeAction::Meme.into_engine(), PokeAction::Meme);
        assert_eq!(CliPokeAction::Nab.into_engine(), PokeAction::Nab);
    }

    #[test]
    fn goose_speak_start_shortcuts_are_uniform() {
        for name in ["honk300", "honk", "goose"] {
            let cli = Cli::try_parse_normalized([name, "plz"]).unwrap();
            assert!(cli.is_start(), "{name} plz should start");
        }
    }

    #[test]
    fn goose_speak_stop_shortcuts() {
        let bad = Cli::try_parse_normalized(["honk", "bad"]).unwrap();
        assert_eq!(bad.command, Some(Command::Stop));
        let no = Cli::try_parse_normalized(["goose", "no"]).unwrap();
        assert_eq!(no.command, Some(Command::Stop));
        let no_honk = Cli::try_parse_normalized(["goose", "no", "honk"]).unwrap();
        assert_eq!(no_honk.command, Some(Command::Stop));
    }

    #[test]
    fn honk_plz_is_start_not_do_honk() {
        let cli = Cli::try_parse_normalized(["honk", "plz"]).unwrap();
        assert_eq!(cli.command, Some(Command::Start));
    }

    #[test]
    fn parses_config_flags_and_lifecycle_commands() {
        let cli = Cli::try_parse_normalized([
            "goose",
            "--config",
            "C:\\tmp\\goose.toml",
            "--wayland",
            "config",
        ])
        .unwrap();
        assert_eq!(cli.command, Some(Command::Config));
        assert!(cli.wayland);
        assert_eq!(cli.config, Some(PathBuf::from("C:\\tmp\\goose.toml")));

        for (word, expected) in [
            ("install", Command::Install),
            ("uninstall", Command::Uninstall),
            ("update", Command::Update),
            ("setup", Command::Setup),
        ] {
            let cli = Cli::try_parse_normalized(["goose", word]).unwrap();
            assert_eq!(cli.command, Some(expected));
        }
    }

    #[test]
    fn explicit_pokes_stay_explicit() {
        let cli = Cli::try_parse_normalized(["goose", "do", "honk"]).unwrap();
        assert_eq!(
            cli.command,
            Some(Command::Do {
                action: CliPokeAction::Honk
            })
        );
    }

    #[test]
    fn help_mentions_goose_speak() {
        let err = Cli::try_parse_normalized(["goose", "--help"]).unwrap_err();
        let help = err.to_string();
        assert!(help.contains("Goose-speak"));
        assert!(help.contains("<name> plz"));
    }

    #[test]
    fn invoked_name_strips_paths_and_extensions() {
        assert_eq!(invoked_name("C:\\Tools\\goose.exe"), "goose");
        assert_eq!(invoked_name("honk300"), "honk300");
    }
}
