use {
    solana_sdk::{message::Message, pubkey::Pubkey, system_instruction, transaction::Transaction},
    std::str::FromStr,
    turnkey::{client::Turnkey, errors::TurnkeyError},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_sign_raw_payload() -> Result<(), TurnkeyError> {
    let turnkey_client = Turnkey::new();

    let pubkey = Pubkey::from_str("3T1uVsZBHiaBVjMMgKog9FEDge5MF2USnDTDHKxBjRYT").unwrap();
    let lamports = 100;
    let instruction = system_instruction::transfer(&pubkey, &pubkey, lamports); // Create an instruction transferring lamports to the same account
    let message = Message::new(&[instruction], Some(&pubkey));
    let mut transaction = Transaction::new_unsigned(message);

    assert!(
        transaction.signatures[0].to_string()
            == "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        "Transaction should not be signed"
    );

    let (tx, sig) = turnkey_client
        .sign_transaction(&mut transaction, &pubkey)
        .await?;

    assert!(
        transaction.signatures[0].to_string()
            != "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        "Transaction should not be signed"
    );

    Ok(())
}

// #[tokio::test]
// async fn test_who_am_i() -> Result<(), TurnkeyError> {
//     let turnkey_client = Turnkey::new();

//     let who_am_i_res = turnkey_client.who_am_i().await.unwrap();

//     println!("{:#?}", who_am_i_res);

//     Ok(())
// }
