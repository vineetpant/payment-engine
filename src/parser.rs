use crate::{errors::PaymentError, types::Transaction};
use csv::{ReaderBuilder, Trim};
use std::io::Read;

/// Parses transactions from a CSV reader asynchronously.
///
/// This function takes a boxed `Read` trait object and returns a boxed iterator
/// over the parsed transactions to iterate over transaction without loading all data in memory upfront.
///  The Reader trims whitespaces from all fields.
///
/// # Arguments
///
/// * `br` - A boxed reader that implements the `Read` trait. This can be a file, stream,
///          or any other readable source.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - On success: A boxed iterator (`Box<dyn Iterator<Item = Result<Transaction, PaymentError>>>`)
/// - On failure: A `PaymentError` detailing the cause of the failure.
pub async fn parse_transactions(
    br: Box<dyn Read>,
) -> Result<Box<dyn Iterator<Item = Result<Transaction, PaymentError>>>, PaymentError> {
    let rdr = ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_reader(br);

    let transactions_iter = rdr
        .into_deserialize()
        .map(|result| result.map_err(|err| PaymentError::CsvParseError(err.to_string())));
    // let mut transactions = Vec::new();
    Ok(Box::new(transactions_iter))
}

#[cfg(test)]
mod tests {
    use crate::{errors::PaymentError, parser::parse_transactions, types::TransactionType};

    #[tokio::test]
    async fn can_parse_csv_stream_and_return_all_transactions() -> Result<(), PaymentError> {
        let csv = "type, client, tx, amount 
        deposit, 1, 1, 1.0 
        deposit, 2, 2, 2.0 
        deposit, 1, 3, 2.0 
        withdrawal, 1, 4, 1.5 
        withdrawal, 2, 5, 3.0";
        let str_buf = stringreader::StringReader::new(csv);
        let transactions = parse_transactions(Box::new(str_buf)).await?;

        assert_eq!(transactions.count(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn can_parse_csv_stream_correctly() -> Result<(), PaymentError> {
        let csv = "type, client, tx, amount 
        deposit, 1, 1, 1.0 
       ";
        let str_buf = stringreader::StringReader::new(csv);
        let mut transactions = parse_transactions(Box::new(str_buf)).await?;

        let fist_transaction = transactions
            .next()
            .ok_or_else(|| PaymentError::CsvParseError("Csv parsing failed".to_string()))??;

        assert_eq!(fist_transaction.r#type, TransactionType::Deposit);
        assert_eq!(fist_transaction.client, 1);
        assert_eq!(fist_transaction.tx, 1);
        assert_eq!(fist_transaction.amount, Some(1.0));
        Ok(())
    }
}
