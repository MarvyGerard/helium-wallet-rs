#[macro_use]
extern crate prettytable;

use helium_wallet::{
    keypair, mnemonic, staking, traits, wallet,
    result};
use std::process;
use structopt::StructOpt;

mod cmd;

use cmd::{balance, create, hotspots, htlc, info, onboard, oui, pay, verify, Opts};

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(flatten)]
    opts: Opts,

    #[structopt(flatten)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Info(info::Cmd),
    Verify(verify::Cmd),
    Balance(balance::Cmd),
    Hotspots(hotspots::Cmd),
    Create(create::Cmd),
    Pay(pay::Cmd),
    Htlc(htlc::Cmd),
    Oui(oui::Cmd),
    Onboard(onboard::Cmd),
}

fn main() {
    let cli = Cli::from_args();
    if let Err(e) = run(cli) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> result::Result {
    match cli.cmd {
        Cmd::Info(cmd) => cmd.run(cli.opts),
        Cmd::Verify(cmd) => cmd.run(cli.opts),
        Cmd::Balance(cmd) => cmd.run(cli.opts),
        Cmd::Hotspots(cmd) => cmd.run(cli.opts),
        Cmd::Create(cmd) => cmd.run(cli.opts),
        Cmd::Pay(cmd) => cmd.run(cli.opts),
        Cmd::Htlc(cmd) => cmd.run(cli.opts),
        Cmd::Oui(cmd) => cmd.run(cli.opts),
        Cmd::Onboard(cmd) => cmd.run(cli.opts),
    }
}
