# payment_engine

## About
A simple payments engine written in Rust that processes a series of transactions from a CSV, updates client accounts, handles disputes, chargebacks, and outputs the final state of clients' accounts. This project is designed to handle large datasets efficiently by streaming CSV transactions instead of loading the entire dataset into memory at once.

## Important Notes
- Streamed CSV Processing: Instead of loading the entire CSV file into memory, transactions are processed as they are read, making it efficient for large datasets.
- Asynchronous processing: Asynchronously parses and processes a given transaction and updates the client’s account state.
- Dispute Handling: Automatically moves disputed funds into a held state and updates the client account state.
- Chargeback Support: Handles chargebacks and locks client accounts when a chargeback occurs.
- Concurrency-Ready: Built with asynchronous functions using tokio for potential future scalability in handling multiple concurrent requests.
- Custom Error Handling: Provides detailed error types for CSV parsing, invalid transactions, and more

## Run the project
You should be able to run your payments engine like

```sh
$ cargo run -- transactions.csv > accounts.csv
```

## Run the tests
You can run simple cargo tests 

```sh
cargo test
```

## Input

The input will be a CSV file with the columns type, client, tx, and amount. You can assume the type is a string, the client column is a valid u16 client ID, the tx is a valid u32 transaction ID, and the amount is a decimal value with a precision of up to four places past the decimal.

For example.

```sh
type, deposit, deposit, deposit, withdrawal, withdrawal,
client, tx, amount 1, 1, 1.0 2, 2, 2.0 1, 3, 2.0 1, 4, 1.5 2, 5, 3.0
```

## Output
The output should be a list of client IDs (client), available amounts (available), held amounts (held), total amounts (total), and whether the account is locked (locked).

For example

```sh
client, available, held, total, locked
1, 1.5, 0.0, 1.5, false
2, 2.0, 0.0, 2.0, false
```