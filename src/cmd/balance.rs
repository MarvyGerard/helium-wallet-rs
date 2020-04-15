use crate::{
    cmd::{api_url, collect_addresses, Opts, OutputFormat},
    result::Result,
};
use helium_api::{Account, Client, Hnt};
use prettytable::{format, Table};
use serde_json::json;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Get the balance for a wallet. The balance is given in HNT and has
/// a precision of 8 decimals.
pub struct Cmd {
    /// Addresses to get balances for
    #[structopt(short = "a", long = "address")]
    addresses: Vec<String>,
}

use super::CmdRunner;
impl CmdRunner for Cmd {
    fn run(&self, opts: Opts) -> Result {
        let client = Client::new_with_base_url(api_url());
        let mut results = Vec::with_capacity(self.addresses.len());
        for address in collect_addresses(opts.files, self.addresses.clone())? {
            results.push((address.to_string(), client.get_account(&address)));
        }
        print_results(results, opts.format);
        Ok(())
    }
}

fn print_results(results: Vec<(String, Result<Account>)>, format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            table.set_titles(row![
                "Address",
                "Balance",
                "Data Credits",
                "Security Tokens"
            ]);
            for (address, result) in results {
                match result {
                    Ok(account) => table.add_row(row![
                        address,
                        Hnt::from_bones(account.balance),
                        account.dc_balance,
                        account.sec_balance
                    ]),
                    Err(err) => table.add_row(row![address, H3 -> err.to_string()]),
                };
            }
            table.printstd();
        }
        OutputFormat::Json => {
            let mut rows = Vec::with_capacity(results.len());
            for (address, result) in results {
                if let Ok(account) = result {
                    let balance = Hnt::from_bones(account.balance).get_decimal();
                    rows.push(json!({
                        "address": address,
                        "dc_balance": account.dc_balance,
                        "sec_balance": account.sec_balance,
                        "balance": balance,
                    }));
                };
            };
            println!("{}", serde_json::to_string_pretty(&rows).unwrap());
        }
    }
}
