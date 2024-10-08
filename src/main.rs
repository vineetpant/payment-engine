mod errors;
mod parser;
mod payment_engine;
mod types;

use std::{fs::File, io::BufReader};

use errors::PaymentError;
use payment_engine::PaymentEngine;

#[tokio::main]
async fn main() -> Result<(), PaymentError> {
    // Get filename from the cli argument
    let file_path = std::env::args().nth(1).ok_or_else(|| {
        PaymentError::InvalidCliArgument("CSV filename missing in cli argument".to_owned())
    })?;

    let br = BufReader::new(
        File::open(file_path).map_err(|err| PaymentError::FileError(err.to_string()))?,
    );
    // Parse the CSV file and get the iterator of transactions
    let transactions = parser::parse_transactions(Box::new(br)).await?;

    // Create a new payment engine and process each transaction
    let mut engine = PaymentEngine::new();

    for txn in transactions {
        engine.process_transaction(txn?).await;
    }

    // Output the final account states to stdout (CSV format)
    engine.output_client_states().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{errors::PaymentError, parser::parse_transactions, payment_engine::PaymentEngine};

    #[tokio::test]
    async fn can_parse_process_print_account_balances_correctly() -> Result<(), PaymentError> {
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

        engine.output_client_states().await;

        Ok(())
    }
}
