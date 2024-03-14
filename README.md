# Turnkey Rust Client
Welcome to the Turnkey Rust Client! This library provides a Rust interface for interacting with the Turnkey API, allowing for the secure storage and signing of transactions via secure enclaves. Let's dive into the essentials to get you started with this powerful tool.

## Overview
The Turnkey Rust Client is designed to simplify the integration with Turnkey API, enabling developers to securely store and sign transactions. Utilizing advanced cryptographic functions and a straightforward API design, this client library is a must-have for developers working with Turnkey services. Whether you're managing keys, signing payloads, or interacting with the blockchain, the Turnkey Rust Client has got you covered.

## Setting Up the Example

### Getting started with the Turnkey Rust Client is easy. Here are the steps to set up your environment and run the example:

### 1. Clone the repository:
First, clone the repository to your local machine:

```bash
git clone https://github.com/Eliascm17/turnkey.git && cd turnkey
```

### 2. Environment Variables:
Before running the client, you'll need to configure a few environment variables. Start by creating a .env file in the root directory of your project:

```bash
touch .env
```

Next, open the .env file and update it with your Turnkey and Helius account details. You can use the .example.env provided in the repository as a template:

```bash
# Helius RPC endpoints
HELIUS_DEVNET_RPC_URL=

# General Turnkey API keys
TURNKEY_ORGANIZATION_ID=
TURNKEY_API_PUBLIC_KEY=
TURNKEY_API_PRIVATE_KEY=

# Example key info
TURNKEY_EXAMPLE_PRIVATE_KEY_ID=
TURNKEY_EXAMPLE_PUBLIC_KEY=
```
Fill in the values as per your Turnkey and Helius account details.

3. Running the Example via `cargo test`

## Usage

```rust
use {
    turnkey::{errors::TurnkeyResult, KeySelector, Turnkey},
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey, system_instruction,
        transaction::Transaction,
    },
    std::{env, str::FromStr},
    dotenv::dotenv,
};

async fn sign_and_submit_transaction() -> TurnkeyResult<()> {
    dotenv().ok();

    // Initialize Turnkey client and RPC client with environment variables
    let turnkey_client = Turnkey::new()?;
    let helius_devnet_rpc_url =
        env::var("HELIUS_DEVNET_RPC_URL").expect("HELIUS_DEVNET_RPC_URL not set");
    let rpc = RpcClient::new_with_commitment(helius_devnet_rpc_url, CommitmentConfig::confirmed());

    // Define public key and lamports for transaction
    let pubkey = Pubkey::from_str(&env::var("TURNKEY_EXAMPLE_PUBLIC_KEY").expect("Public key not set"))
        .expect("Invalid public key format");
    let lamports = 100;

    // Create a basic transfer instruction and message with a recent blockhash
    let instruction = system_instruction::transfer(&pubkey, &pubkey, lamports);
    let recent_blockhash = rpc
        .get_latest_blockhash()
        .expect("Failed to get latest blockhash");
    let message = Message::new_with_blockhash(&[instruction], Some(&pubkey), &recent_blockhash);
    let mut transaction = Transaction::new_unsigned(message);

    // Sign the transaction with the Turnkey client
    let (signed_tx, _signature) = turnkey_client
        .sign_transaction(&mut transaction, KeySelector::ExampleKey)
        .await?;

    // Submit the signed transaction
    rpc.send_and_confirm_transaction(&signed_tx)
        .expect("Failed to send and confirm transaction");

    Ok(())
}
```

This example demonstrates initializing the Turnkey client, preparing a transaction, signing it, and submitting it to the blockchain. Modify and extend it according to your specific needs.

For more information and detailed documentation, please refer to the official [Turnkey documentation](https://docs.turnkey.com/).

Feel free to submit any issues or pull requests to this repository. Happy coding!
