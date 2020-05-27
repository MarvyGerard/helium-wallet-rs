use crate::{
    cmd::{api_url, get_password, get_payer, load_wallet, Opts, OutputFormat},
    keypair::PubKeyBin,
    result::Result,
    staking,
    traits::{Sign, Signer, TxnEnvelope, B58, B64},
};
use helium_api::{Client, PendingTxnStatus};
use helium_proto::{BlockchainTxn, BlockchainTxnOuiV1, Txn};
use serde_json::json;
use structopt::StructOpt;

/// Create or update an OUI
#[derive(Debug, StructOpt)]
pub enum Cmd {
    Create(Create),
    Submit(Submit),
}

/// Allocates an Organizational Unique Identifier (OUI) which
/// identifies endpoints for packets to sent to The transaction is not
/// submitted to the system unless the '--commit' option is given.
#[derive(Debug, StructOpt)]
pub struct Create {
    /// The address(es) of the router to send packets to
    #[structopt(long = "address", short = "a", number_of_values(1))]
    addresses: Vec<PubKeyBin>,

    /// Initial device membership filter in base64 encoded form
    #[structopt(long)]
    filter: String,

    /// The requested OUI. This needs to be one larger than the
    /// current OUI on the blockchain.
    #[structopt(long)]
    oui: u64,

    /// Requested subnet size. Must be a value between 8 and 65,536
    /// and a power of two.
    #[structopt(long)]
    subnet_size: u32,

    /// Payer for the transaction (B58 address). If not specified the
    /// wallet is used. If "staking" is used the Helium staking server
    /// is used as the payer.
    #[structopt(long)]
    payer: Option<String>,

    /// Commit the transaction to the API. If the staking server is
    /// used as the payer the transaction must first be submitted to
    /// the staking server for signing and the result submitted ot the
    /// API.
    #[structopt(long)]
    commit: bool,
}

/// Submits a given base64 oui transaction to the API. This command
/// can be used when this wallet is not the payer of the oui
/// transaction.
#[derive(Debug, StructOpt)]
pub struct Submit {
    /// Base64 encoded transaction to submit.
    #[structopt(name = "TRANSACTION")]
    transaction: String,

    /// Commit the payment to the API. If the staking server is used
    /// as the payer the transaction is first submitted to the staking
    /// server for signing and the result submitted ot the API.
    #[structopt(long)]
    commit: bool,
}

impl Cmd {
    pub fn run(&self, opts: Opts) -> Result {
        match self {
            Cmd::Create(cmd) => cmd.run(opts),
            Cmd::Submit(cmd) => cmd.run(opts),
        }
    }
}

impl Create {
    pub fn run(&self, opts: Opts) -> Result {
        let password = get_password(false)?;
        let wallet = load_wallet(opts.files)?;
        let keypair = wallet.to_keypair(password.as_bytes())?;

        let api_client = Client::new_with_base_url(api_url());
        let staking_client = staking::Client::default();

        let staking_key = staking_client.address()?;
        let wallet_key = keypair.pubkey_bin();

        let payer = get_payer(staking_key, &self.payer)?;

        let mut txn = BlockchainTxnOuiV1 {
            addresses: self
                .addresses
                .clone()
                .into_iter()
                .map(|s| s.to_vec())
                .collect(),
            owner: keypair.pubkey_bin().into(),
            payer: payer.map_or(vec![], |v| v.to_vec()),
            oui: self.oui,
            fee: 0,
            staking_fee: 1,
            owner_signature: vec![],
            payer_signature: vec![],
            requested_subnet_size: self.subnet_size,
            filter: base64::decode(&self.filter)?,
        };

        let envelope = txn.sign(&keypair, Signer::Owner)?.in_envelope();

        match payer {
            key if key == Some(wallet_key) || key.is_none() => {
                // Payer is the wallet submit if ready to commit
                let status = if self.commit {
                    Some(api_client.submit_txn(&envelope)?)
                } else {
                    None
                };
                print_txn(&txn, &envelope, &status, opts.format)
            }
            _ => {
                // Payer is either staking server or something else.
                // can't commit this transaction but we can display it
                print_txn(&txn, &envelope, &None, opts.format)
            }
        }
    }
}

impl Submit {
    pub fn run(&self, opts: Opts) -> Result {
        let envelope = BlockchainTxn::from_b64(&self.transaction)?;
        if let Some(Txn::Oui(t)) = envelope.txn.clone() {
            let api_client = helium_api::Client::new_with_base_url(api_url());
            let status = if self.commit {
                Some(api_client.submit_txn(&envelope)?)
            } else {
                None
            };
            print_txn(&t, &envelope, &status, opts.format)
        } else {
            Err("Invalid OUI transaction".into())
        }
    }
}

fn print_txn(
    txn: &BlockchainTxnOuiV1,
    envelope: &BlockchainTxn,
    status: &Option<PendingTxnStatus>,
    format: OutputFormat,
) -> Result {
    match format {
        OutputFormat::Table => {
            ptable!(
                ["Reqeuested Subnet Size", "Addresses"],
                [
                    txn.requested_subnet_size,
                    txn.addresses
                        .clone()
                        .into_iter()
                        .map(|v| PubKeyBin::from_vec(&v).to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                ]
            );

            if status.is_some() {
                ptable!(["Hash"], [status.as_ref().map_or("none", |s| &s.hash)]);
            }

            Ok(())
        }
        OutputFormat::Json => {
            let table = json!({
                "addresses": txn.addresses
                    .clone()
                    .into_iter()
                    .map(|v| PubKeyBin::from_vec(&v).to_string())
                    .collect::<Vec<String>>(),
                "requested_subnet_size": txn.requested_subnet_size,
                "payer": PubKeyBin::from_vec(&txn.payer).to_b58().unwrap(),
                "hash": status.as_ref().map(|s| &s.hash),
                "txn": envelope.to_b64()?,
            });

            println!("{}", serde_json::to_string_pretty(&table)?);
            Ok(())
        }
    }
}
