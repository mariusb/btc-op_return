# btc-op_return

A command-line tool for fetching and displaying OP_RETURN data from Bitcoin transactions in a given block.

## Description

This tool connects to the mempool.space API to retrieve transaction data for a specified Bitcoin block. It then iterates through all transactions in that block, identifies any `OP_RETURN` outputs, and prints the associated data in both hexadecimal and ASCII formats. Finally, it provides some basic statistics about the OP_RETURN data found in the block.

## Usage

### Prerequisites

- Rust and Cargo must be installed.

### Building

To build the project, run the following command in the project's root directory:

```bash
cargo build --release
```

### Running

To run the tool, use the following command:

```bash
cargo run --release [block_number]
```

If no block number is provided, the tool will automatically fetch data for the latest block.

## Dependencies

This project uses the following Rust crates:

- `reqwest`
- `serde`
- `serde_json`
- `tokio`
- `hex`
