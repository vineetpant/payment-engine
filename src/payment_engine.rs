use crate::types::{Client, Transaction, TransactionType};
use std::collections::HashMap;

pub struct PaymentEngine {
    pub clients: HashMap<u16, Client>,
    pub transactions: HashMap<u32, Transaction>,
    pub disputed_transactions: HashMap<u32, Transaction>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        PaymentEngine {
            clients: HashMap::new(),
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
        }
    }

    /// Asynchronously processes a given transaction and updates the client’s account state.
    ///
    /// # Arguments
    ///
    /// * `txn` - A `Transaction` object representing the incoming transaction to be processed.
    ///           This contains the type of transaction and associated metadata (e.g., client ID, amount).
    ///
    /// # Transaction Types
    ///
    /// * `Deposit`: Increases the client’s available funds and total balance.
    /// * `Withdrawal`: Decreases the client’s available funds, if sufficient funds are present.
    /// * `Dispute`: Temporarily moves funds from available to held, pending a resolution.
    /// * `Resolve`: Moves held funds back to available, resolving the dispute.
    /// * `Chargeback`: Finalizes a dispute, removing held funds and locking the client’s account.
    pub async fn process_transaction(&mut self, txn: Transaction) {
        match txn.r#type {
            TransactionType::Deposit => self.process_deposit(txn),
            TransactionType::Withdrawal => self.process_withdrawal(txn),
            TransactionType::Dispute => self.process_dispute(txn),
            TransactionType::Resolve => self.process_resolve(txn),
            TransactionType::Chargeback => self.process_chargeback(txn),
        }
    }

    fn process_deposit(&mut self, txn: Transaction) {
        let client = self.clients.entry(txn.client).or_insert(Client::new());

        if !client.locked {// don't process if account is locked
            if let Some(amount) = txn.amount {
                client.available += amount;
                client.total += amount;
            }
            self.transactions.insert(txn.tx, txn);
        }
    }

    fn process_withdrawal(&mut self, txn: Transaction) {
        let client = self.clients.get_mut(&txn.client);
        if let Some(client) = client {
            if !client.locked { // don't process if account is locked
                if let Some(amount) = txn.amount {
                    if client.available >= amount {
                        client.available -= amount;
                        client.total -= amount;

                        self.transactions.insert(txn.tx, txn);
                    }
                }
            }
        }
    }

    fn process_dispute(&mut self, txn: Transaction) {
        if let Some(original_txn) = self.transactions.get(&txn.tx) {
            if original_txn.client == txn.client { // both transaction should refer to same client
                let client = self.clients.get_mut(&original_txn.client);
                if let Some(client) = client {
                    if let Some(amount) = original_txn.amount {
                        client.available -= amount;
                        client.held += amount;
                    }
                }
                self.disputed_transactions.insert(txn.tx, txn);
            }
        }
    }

    fn process_resolve(&mut self, txn: Transaction) {
        if self.disputed_transactions.contains_key(&txn.tx) { // resolve only if disputed transaction reference is present
            if let Some(original_txn) = self.transactions.get(&txn.tx) {
                if original_txn.client == txn.client { // both transaction should refer to same client
                    let client = self.clients.get_mut(&original_txn.client);
                    if let Some(client) = client {
                        if let Some(amount) = original_txn.amount {
                            client.available += amount;
                            client.held -= amount;
                        }
                    }
                }
            }
        }
    }

    fn process_chargeback(&mut self, txn: Transaction) {
        if self.disputed_transactions.contains_key(&txn.tx) { // chargeback only if disputed transaction reference is present
            if let Some(original_txn) = self.transactions.get(&txn.tx) {
                if original_txn.client == txn.client { // both transaction should refer to same client
                    let client = self.clients.get_mut(&original_txn.client);
                    if let Some(client) = client {
                        if let Some(amount) = original_txn.amount {
                            client.total -= amount;
                            if client.available < 0.0 || client.total < 0.0 {
                                client.total = 0.0;
                                client.available = 0.0;
                            }
                            client.held -= amount;
                            client.locked = true;
                        }
                    }
                }
            }
        }
    }

    /// This asynchronous function prints the state of each client in a CSV format, including the
    /// available funds, held funds, total balance, and account lock status.
    ///
    /// The function outputs the state of each client as follows:
    ///
    /// ```text
    /// client,available,held,total,locked
    /// 1,100.0000,0.0000,100.0000,false
    /// 2,200.0000,0.0000,200.0000,false
    /// ```
    ///
    /// The available, held, and total values are displayed with four decimal places.
    pub async fn output_client_states(&self) {
        println!("client,available,held,total,locked");
        for (client_id, client) in &self.clients {
            println!(
                "{},{:.4},{:.4},{:.4},{}",
                client_id, client.available, client.held, client.total, client.locked
            );
        }
    }
}

// Test trasaction processor
#[cfg(test)]
mod tests {
    use crate::{errors::PaymentError, parser::parse_transactions, payment_engine::PaymentEngine};

    #[tokio::test]
    async fn can_process_simple_transactions() -> Result<(), PaymentError> {
        let csv = "type, client, tx, amount 
        deposit, 1, 1, 1.0 
        deposit, 2, 2, 2.0 
        deposit, 1, 3, 2.0 
        withdrawal, 1, 4, 1.5 
        withdrawal, 2, 5, 3.0";
        let str_buf = stringreader::StringReader::new(csv);
        let transactions = parse_transactions(Box::new(str_buf)).await?;
        let mut engine = PaymentEngine::new();

        for txn in transactions {
            engine.process_transaction(txn?).await;
        }

        if let Some(client) = engine.clients.get(&1) {
            assert_eq!(client.total, 1.5);
            assert_eq!(client.available, 1.5);
            assert!(!client.locked);
            assert_eq!(client.held, 0.0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_process_chargeback_transactions() -> Result<(), PaymentError> {
        let csv = "type, client, tx, amount
        deposit, 1, 1, 1.0
        deposit, 2, 2, 2.0
        deposit, 1, 3, 2.0
        withdrawal, 1, 4, 1.5
        withdrawal, 2, 5, 3.0
        dispute, 1, 3
        resolve, 1, 3
        dispute, 2, 2
        chargeback, 2, 2";
        let str_buf = stringreader::StringReader::new(csv);
        let transactions = parse_transactions(Box::new(str_buf)).await?;
        let mut engine = PaymentEngine::new();

        for txn in transactions {
            engine.process_transaction(txn?).await;
        }

        if let Some(client) = engine.clients.get(&2) {
            assert_eq!(client.total, 0.0);
            assert_eq!(client.available, 0.0);
            assert!(client.locked); // should be locked due to chargeback
            assert_eq!(client.held, 0.0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_process_disputed_transactions() -> Result<(), PaymentError> {
        let csv = "type, client, tx, amount
        deposit, 1, 1, 1.0
        deposit, 2, 2, 2.0
        deposit, 1, 3, 2.0
        withdrawal, 1, 4, 1.5
        withdrawal, 2, 5, 3.0
        dispute, 2, 2";

        let str_buf = stringreader::StringReader::new(csv);
        let transactions = parse_transactions(Box::new(str_buf)).await?;
        let mut engine = PaymentEngine::new();

        for txn in transactions {
            engine.process_transaction(txn?).await;
        }

        if let Some(client) = engine.clients.get(&2) {
            assert_eq!(client.total, 2.0);
            assert_eq!(client.available, 0.0); // available should be 0 due to dispute
            assert!(!client.locked);
            assert_eq!(client.held, 2.0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_process_resolved_transactions() -> Result<(), PaymentError> {
        let csv = "type, client, tx, amount
        deposit, 1, 1, 1.0
        deposit, 2, 2, 2.0
        deposit, 1, 3, 2.0
        withdrawal, 1, 4, 1.5
        withdrawal, 2, 5, 3.0
        dispute, 2, 2,
        resolve, 2, 2";

        let str_buf = stringreader::StringReader::new(csv);
        let transactions = parse_transactions(Box::new(str_buf)).await?;
        let mut engine = PaymentEngine::new();

        for txn in transactions {
            engine.process_transaction(txn?).await;
        }

        if let Some(client) = engine.clients.get(&2) {
            assert_eq!(client.total, 2.0);
            assert_eq!(client.available, 2.0);
            assert!(!client.locked);
            assert_eq!(client.held, 0.0); // held should be 0 as dispute is resolved
        }

        Ok(())
    }
}
