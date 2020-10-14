use crate::{
    keypair::PubKeyBin,
    result::Result,
    traits::{B58, B64},
};
use helium_api::{BlockchainTxnVarsV1, BlockchainVarV1};

pub(crate) fn maybe_b58(data: &[u8]) -> Result<Option<String>> {
    if data.is_empty() {
        Ok(None)
    } else {
        Ok(Some(data.to_vec().to_b58()?))
    }
}

pub(crate) fn maybe_b64_url(data: &[u8]) -> Result<Option<String>> {
    if data.is_empty() {
        Ok(None)
    } else {
        Ok(Some(data.to_vec().to_b64_url()?))
    }
}

pub trait ToJson {
    fn to_json(&self) -> Result<serde_json::Value>;
}

impl<T> ToJson for Vec<T>
    where
        T: ToJson,
{
    fn to_json(&self) -> Result<serde_json::Value> {
        let mut seq = Vec::with_capacity(self.len());
        for entry in self {
            seq.push(entry.to_json()?)
        }
        Ok(json!(seq))
    }
}

fn vec_to_strings(vec: &[Vec<u8>]) -> Result<Vec<String>> {
    let mut seq = Vec::with_capacity(vec.len());
    for entry in vec {
        seq.push(String::from_utf8(entry.to_vec())?);
    }
    Ok(seq)
}

fn vec_to_b58s(vec: &[Vec<u8>]) -> Result<Vec<String>> {
    let mut seq = Vec::with_capacity(vec.len());
    for entry in vec {
        seq.push(PubKeyBin::from_vec(entry).to_b58()?);
    }
    Ok(seq)
}

fn vec_to_b64_urls(vec: &[Vec<u8>]) -> Result<Vec<String>> {
    let mut seq = Vec::with_capacity(vec.len());
    for entry in vec {
        seq.push(entry.to_b64_url()?);
    }
    Ok(seq)
}

impl ToJson for BlockchainTxnVarsV1 {
    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::Value::String(serde_json::to_string(self).unwrap()))
    }
}

impl ToJson for BlockchainVarV1 {
    fn to_json(&self) -> Result<serde_json::Value> {
        let value = match &self.r#type[..] {
            "int" => {
                let v: i64 = String::from_utf8(self.value.to_vec())?.parse::<i64>()?;
                json!(v)
            }
            "float" => {
                let v: f64 = String::from_utf8(self.value.to_vec())?.parse::<f64>()?;
                json!(v)
            }
            "string" => {
                let v: String = String::from_utf8(self.value.to_vec())?;
                json!(v)
            }
            _ => return Err(format!("Invalid variable {:?}", self).into()),
        };
        Ok(json!({
            "name": self.name,
            "value": value
        }))
    }
}