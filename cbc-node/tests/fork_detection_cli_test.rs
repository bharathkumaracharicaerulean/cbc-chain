use clap::Parser;
use cbc_node::cli::{Cli, Subcommand};

#[test]
fn test_fork_check_cli_help() {
    let res = Cli::try_parse_from(&["cbc-node", "fork-check", "--help"]);
    assert!(res.is_err(), "DisplayHelp returns Err with ErrorKind::DisplayHelp");
    let err = res.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    
    let help_msg = err.to_string();
    assert!(help_msg.contains("Fork detection tool"));
    assert!(help_msg.contains("--local-rpc"));
    assert!(help_msg.contains("--peer-rpc"));
    assert!(help_msg.contains("--threshold"));
    assert!(help_msg.contains("--format"));
    assert!(help_msg.contains("--timeout"));
}

#[test]
fn test_fork_check_cli_validation() {
    let cli = Cli::try_parse_from(&[
        "cbc-node",
        "fork-check",
        "--local-rpc", "http://localhost:9944",
        "--peer-rpc", "http://peer1:9944,http://peer2:9944",
        "--threshold", "15",
        "--timeout", "45",
        "--format", "json",
    ]).expect("CLI parsing should succeed");

    if let Some(Subcommand::ForkCheck(cmd)) = cli.subcommand {
        assert_eq!(cmd.local_rpc, "http://localhost:9944");
        assert_eq!(cmd.peer_rpc, vec!["http://peer1:9944", "http://peer2:9944"]);
        assert_eq!(cmd.threshold, 15);
        assert_eq!(cmd.timeout, 45);
    } else {
        panic!("Parsed subcommand should be ForkCheck");
    }
}