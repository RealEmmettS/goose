use clap::{Args, Parser, Subcommand, ValueEnum};
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
    after_help = "Goose-speak:\n  <name> plz                         Start the goose\n  <name> bad | no | no honk          Walk offscreen, then stop\n  <name> exit | quit                 Walk offscreen, then stop\n  <name> stop | exit | quit --force  Stop immediately\n  <name> do honk                     Poke a honk\n\nInstalled names: honk300, honk, goose."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Args)]
pub struct StartOptions {
    /// Start the goose muted.
    #[arg(long, alias = "silent")]
    pub no_sound: bool,

    /// Disable cursor-stealing behavior for this run.
    #[arg(long)]
    pub no_mouse_steal: bool,

    /// Disable foreign-window ride behavior for this run.
    #[arg(long)]
    pub no_window_ride: bool,

    /// Use a specific config.toml instead of the per-user default.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Request native Wayland reduced mode on Linux.
    #[arg(long)]
    pub wayland: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Start the goose. This is also the default when no command is provided.
    Start {
        #[command(flatten)]
        options: StartOptions,
    },
    /// Stop the running goose through local IPC.
    Stop {
        /// Stop immediately instead of walking offscreen first.
        #[arg(long)]
        force: bool,
    },
    /// Ask the running goose to reload runtime options.
    Reload,
    /// Show the running goose's platform and capability status.
    Status,
    /// Open the terminal config editor.
    Config {
        /// Use a specific config.toml instead of the per-user default.
        #[arg(long, value_name = "PATH")]
        config: Option<PathBuf>,
    },
    /// Poke the running goose into a specific action.
    Do {
        #[arg(value_enum)]
        action: CliPokeAction,
    },
    /// Install honk300 assets, aliases, desktop shortcuts, and optional autostart.
    Install {
        /// Start honk300 when the user logs in.
        #[arg(long)]
        autostart: bool,
    },
    /// Remove installed honk300 aliases, shortcuts, markers, and optionally user state.
    Uninstall {
        /// Back up user memes/notes, then remove config/state as well as app files.
        #[arg(long)]
        purge: bool,
    },
    /// Download and run the matching release installer for this install source.
    Update {
        /// Emit exactly one machine-readable result object on stdout.
        #[arg(long)]
        json: bool,
    },
    /// Create or refresh the user config file.
    Setup {
        /// Use a specific config.toml instead of the per-user default.
        #[arg(long, value_name = "PATH")]
        config: Option<PathBuf>,
        /// Back up the existing file and replace it with schema-current defaults.
        #[arg(long)]
        reset: bool,
    },
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
        self.command.is_none() || matches!(self.command, Some(Command::Start { .. }))
    }

    pub fn is_client_command(&self) -> bool {
        matches!(
            self.command,
            Some(Command::Stop { .. } | Command::Reload | Command::Status | Command::Do { .. })
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
        if args.len() > 1 {
            args.insert(1, "start".into());
        }
        return args;
    }

    let token = args[command_idx].to_ascii_lowercase();
    match token.as_str() {
        "plz" => args[command_idx] = "start".into(),
        // Natural stop synonyms: `goose exit` / `goose quit` behave exactly like stop.
        "exit" | "quit" => args[command_idx] = "stop".into(),
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

    if command_idx > 1 && matches!(args[command_idx].as_str(), "start" | "config" | "setup") {
        let prefix: Vec<_> = args.drain(1..command_idx).collect();
        args.splice(2..2, prefix);
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
    fn parses_stop_reload_status_and_do() {
        let stop = Cli::try_parse_normalized(["honk300", "stop"]).unwrap();
        assert_eq!(stop.command, Some(Command::Stop { force: false }));

        let forced_stop = Cli::try_parse_normalized(["honk300", "stop", "--force"]).unwrap();
        assert_eq!(forced_stop.command, Some(Command::Stop { force: true }));

        let reload = Cli::try_parse_normalized(["honk300", "reload"]).unwrap();
        assert_eq!(reload.command, Some(Command::Reload));

        let status = Cli::try_parse_normalized(["honk300", "status"]).unwrap();
        assert_eq!(status.command, Some(Command::Status));

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
        assert_eq!(bad.command, Some(Command::Stop { force: false }));
        let no = Cli::try_parse_normalized(["goose", "no"]).unwrap();
        assert_eq!(no.command, Some(Command::Stop { force: false }));
        let no_honk = Cli::try_parse_normalized(["goose", "no", "honk"]).unwrap();
        assert_eq!(no_honk.command, Some(Command::Stop { force: false }));
        let exit = Cli::try_parse_normalized(["goose", "exit"]).unwrap();
        assert_eq!(exit.command, Some(Command::Stop { force: false }));
        let quit = Cli::try_parse_normalized(["honk", "quit"]).unwrap();
        assert_eq!(quit.command, Some(Command::Stop { force: false }));

        for word in ["stop", "exit", "quit", "bad", "no"] {
            let forced = Cli::try_parse_normalized(["goose", word, "--force"]).unwrap();
            assert_eq!(
                forced.command,
                Some(Command::Stop { force: true }),
                "goose {word} --force should stop immediately"
            );
        }

        let forced_no_honk = Cli::try_parse_normalized(["goose", "no", "honk", "--force"]).unwrap();
        assert_eq!(forced_no_honk.command, Some(Command::Stop { force: true }));
    }

    #[test]
    fn all_three_names_are_fully_interchangeable() {
        // Every command form the CLI accepts, as the argument tail after the program name.
        // Parsing must be identical whether the goose was invoked as honk300, honk, or goose.
        let tails: &[&[&str]] = &[
            &[],
            &["start"],
            &["plz"],
            &["stop"],
            &["stop", "--force"],
            &["bad"],
            &["no"],
            &["no", "honk"],
            &["exit"],
            &["quit"],
            &["quit", "--force"],
            &["reload"],
            &["status"],
            &["config"],
            &["setup"],
            &["update"],
            &["do", "honk"],
            &["do", "wander"],
            &["do", "mud"],
            &["do", "meme"],
            &["do", "note"],
            &["do", "nab"],
            &["install"],
            &["install", "--autostart"],
            &["uninstall"],
            &["uninstall", "--purge"],
            &["--no-sound", "start"],
            &["--no-mouse-steal", "start"],
            &["--no-window-ride", "start"],
            &["--wayland", "start"],
            &["--config", "C:\\tmp\\g.toml", "config"],
            &["setup", "--config", "C:\\tmp\\g.toml", "--reset"],
        ];

        for tail in tails {
            let baseline_args: Vec<&str> = std::iter::once("honk300")
                .chain(tail.iter().copied())
                .collect();
            let baseline = Cli::try_parse_normalized(baseline_args)
                .unwrap_or_else(|e| panic!("honk300 {tail:?} should parse: {e}"));

            for name in ["honk", "goose"] {
                let args: Vec<&str> = std::iter::once(name).chain(tail.iter().copied()).collect();
                let cli = Cli::try_parse_normalized(args)
                    .unwrap_or_else(|e| panic!("{name} {tail:?} should parse: {e}"));
                assert_eq!(
                    cli.command, baseline.command,
                    "`{name} {tail:?}` command differs"
                );
            }
        }
    }

    #[test]
    fn honk_plz_is_start_not_do_honk() {
        let cli = Cli::try_parse_normalized(["honk", "plz"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Start { .. })));
    }

    #[test]
    fn parses_config_flags_and_lifecycle_commands() {
        let cli = Cli::try_parse_normalized(["goose", "config", "--config", "C:\\tmp\\goose.toml"])
            .unwrap();
        assert_eq!(
            cli.command,
            Some(Command::Config {
                config: Some(PathBuf::from("C:\\tmp\\goose.toml")),
            })
        );

        for (word, expected) in [
            ("install", Command::Install { autostart: false }),
            ("uninstall", Command::Uninstall { purge: false }),
            ("update", Command::Update { json: false }),
            (
                "setup",
                Command::Setup {
                    config: None,
                    reset: false,
                },
            ),
            ("status", Command::Status),
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
    fn parses_lifecycle_flags() {
        let install = Cli::try_parse_normalized(["honk300", "install", "--autostart"]).unwrap();
        assert_eq!(install.command, Some(Command::Install { autostart: true }));

        let uninstall = Cli::try_parse_normalized(["honk300", "uninstall", "--purge"]).unwrap();
        assert_eq!(uninstall.command, Some(Command::Uninstall { purge: true }));

        let update = Cli::try_parse_normalized(["honk300", "update", "--json"]).unwrap();
        assert_eq!(update.command, Some(Command::Update { json: true }));
    }

    #[test]
    fn runtime_overrides_are_scoped_to_start() {
        for flag in [
            "--no-sound",
            "--silent",
            "--no-mouse-steal",
            "--no-window-ride",
            "--wayland",
        ] {
            assert!(
                Cli::try_parse_normalized(["honk300", "start", flag]).is_ok(),
                "{flag} should be accepted by start"
            );
            assert!(
                Cli::try_parse_normalized(["honk300", "status", flag]).is_err(),
                "{flag} must be rejected by status"
            );
            assert!(
                Cli::try_parse_normalized(["honk300", flag, "status"]).is_err(),
                "a pre-command {flag} must not leak into status"
            );
        }
    }

    #[test]
    fn config_path_is_scoped_to_start_config_and_setup() {
        for command in ["start", "config", "setup"] {
            assert!(
                Cli::try_parse_normalized(["honk300", command, "--config", "custom.toml"]).is_ok(),
                "{command} should accept --config"
            );
        }
        for command in ["stop", "reload", "status", "update"] {
            assert!(
                Cli::try_parse_normalized(["honk300", command, "--config", "custom.toml"]).is_err(),
                "{command} must reject unused --config"
            );
        }
    }

    #[test]
    fn setup_accepts_explicit_reset_only_on_setup() {
        let setup =
            Cli::try_parse_normalized(["honk300", "setup", "--config", "custom.toml", "--reset"])
                .unwrap();
        assert_eq!(
            setup.command,
            Some(Command::Setup {
                config: Some(PathBuf::from("custom.toml")),
                reset: true,
            })
        );
        assert!(Cli::try_parse_normalized(["honk300", "start", "--reset"]).is_err());
    }

    #[test]
    fn default_start_keeps_pre_command_aliases_and_overrides() {
        let cli =
            Cli::try_parse_normalized(["goose", "--silent", "--config", "goose.toml"]).unwrap();
        let Some(Command::Start { options }) = cli.command else {
            panic!("start options should materialize an explicit normalized start command");
        };
        assert!(options.no_sound);
        assert_eq!(options.config, Some(PathBuf::from("goose.toml")));

        let cli = Cli::try_parse_normalized(["goose", "--silent", "plz"]).unwrap();
        assert!(cli.is_start());
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
        #[cfg(windows)]
        assert_eq!(invoked_name("C:\\Tools\\goose.exe"), "goose");

        #[cfg(not(windows))]
        assert_eq!(invoked_name("/opt/honk/goose"), "goose");

        assert_eq!(invoked_name("honk300"), "honk300");
    }
}
