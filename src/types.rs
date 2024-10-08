use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

pub struct Client {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Client {
    pub fn new() -> Self {
        Client {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}
