use clap::Parser;

/// CLI arguments — mirrors `parseArgs` in main.ts
/// Supports `-y/--yes --dry-run --clear-cache -a/--agent -v/--verbose -h/--help`
#[derive(Debug, Parser, Clone)]
#[command(
    name = "skillscout",
    version = env!("CARGO_PKG_VERSION"),
    about = "Auto-install the best AI skills for your project",
    disable_help_flag = false
)]
pub struct Args {
    /// Skip confirmation prompt
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// Show what would be installed without installing
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Clear downloaded skills cache
    #[arg(long = "clear-cache")]
    pub clear_cache: bool,

    /// Install for specific IDEs only (e.g. cursor, claude-code)
    #[arg(short = 'a', long = "agent", value_name = "AGENT", num_args = 1..)]
    pub agent: Vec<String>,

    /// Show install trace and error details
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}

impl Args {
    /// Parse from current process args (like Clap's `parse`)
    pub fn parse_from_env() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn parse<I, T>(args: I) -> Args
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Args::try_parse_from(args).expect("parse failed")
    }

    #[test]
    fn flags_scenario() {
        let args = parse([
            "skillscout",
            "-y",
            "--dry-run",
            "-a",
            "cursor",
            "claude-code",
            "-v",
        ]);
        assert!(args.yes, "autoYes should be true");
        assert!(args.dry_run);
        assert_eq!(args.agent, vec!["cursor", "claude-code"]);
        assert!(args.verbose);
    }

    #[test]
    fn long_forms() {
        let args = parse([
            "skillscout",
            "--yes",
            "--dry-run",
            "--agent",
            "cursor",
            "--verbose",
        ]);
        assert!(args.yes);
        assert!(args.dry_run);
        assert_eq!(args.agent, vec!["cursor"]);
        assert!(args.verbose);
    }

    #[test]
    fn clear_cache_flag() {
        let args = parse(["skillscout", "--clear-cache"]);
        assert!(args.clear_cache);
        assert!(!args.yes);
        assert!(!args.dry_run);
    }

    #[test]
    fn agent_multiple_values_after_single_flag() {
        // `-a cursor claude-code` should collect both after one -a (num_args=1..)
        let args = parse(["skillscout", "-a", "cursor", "claude-code"]);
        assert_eq!(args.agent, vec!["cursor", "claude-code"]);
    }

    #[test]
    fn agent_repeated_flag() {
        let args = parse(["skillscout", "-a", "cursor", "-a", "claude-code"]);
        assert_eq!(args.agent, vec!["cursor", "claude-code"]);
    }

    #[test]
    fn agent_mixed_with_other_flags() {
        let args = parse([
            "skillscout",
            "-a",
            "cursor",
            "claude-code",
            "-v",
            "--dry-run",
        ]);
        assert_eq!(args.agent, vec!["cursor", "claude-code"]);
        assert!(args.verbose);
        assert!(args.dry_run);
    }

    #[test]
    fn no_agents_empty_vec() {
        let args = parse(["skillscout"]);
        assert!(args.agent.is_empty());
    }

    #[test]
    fn help_flag_generates_help() {
        let cmd = Args::command();
        let matches = cmd.try_get_matches_from(["skillscout", "--help"]);
        // Clap returns Err with DisplayHelp on --help
        assert!(matches.is_err());
        let err = matches.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn short_help_flag() {
        let cmd = Args::command();
        let err = cmd.try_get_matches_from(["skillscout", "-h"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn verbose_short_long() {
        let a = parse(["skillscout", "-v"]);
        assert!(a.verbose);
        let b = parse(["skillscout", "--verbose"]);
        assert!(b.verbose);
    }
}
