use {
    dotenv::dotenv,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey, system_instruction,
        transaction::Transaction,
    },
    std::{env, str::FromStr},
    turnkey::{errors::TurnkeyResult, KeySelector, Turnkey},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_example_sign_raw_payload() -> TurnkeyResult<()> {
    dotenv().ok();

    // Initialize Turnkey client and RPC client with environment variables
    let turnkey_client = Turnkey::new()?;
    let helius_devnet_rpc_url =
        env::var("HELIUS_DEVNET_RPC_URL").expect("HELIUS_DEVNET_RPC_URL not set");
    let rpc = RpcClient::new_with_commitment(helius_devnet_rpc_url, CommitmentConfig::confirmed());

    // Define public key and lamports for transaction
    let pubkey =
        Pubkey::from_str(&env::var("TURNKEY_EXAMPLE_PUBLIC_KEY").expect("Public key not set"))
            .expect("Invalid public key format");
    let lamports = 100;

    // Create a basic transfer instruction and message with a recent blockhash
    let instruction = system_instruction::transfer(&pubkey, &pubkey, lamports);
    let recent_blockhash = rpc
        .get_latest_blockhash()
        .expect("Failed to get latest blockhash");
    let message = Message::new_with_blockhash(&[instruction], Some(&pubkey), &recent_blockhash);
    let mut transaction = Transaction::new_unsigned(message);

    // Check for default transaction signature before signing
    // Expected: The default signature indicates an unsigned transaction
    assert_eq!(
        transaction.signatures[0].to_string(),
        "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        "Expected the default signature before signing."
    );

    // Sign the transaction with the Turnkey client
    let (tx, _sig) = turnkey_client
        .sign_transaction(&mut transaction, KeySelector::ExampleKey)
        .await?;

    // Send the transaction and confirm it
    // This also acts as a runtime check to ensure the transaction was successfully signed and accepted
    let tx_sig = rpc
        .send_and_confirm_transaction(&tx)
        .expect("Failed to send and confirm transaction");

    // Verify the transaction signature has been updated to the actual signature
    // Expected: The transaction's first signature should match the returned transaction signature
    assert_eq!(
        transaction.signatures[0].to_string(),
        tx_sig.to_string(),
        "Expected the transaction signature to be updated after signing."
    );

    Ok(())
}
