use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
};
use solana_system_interface::{instruction::transfer, program as system_program};
use bs58;
use std::io::{self, BufRead};
use std::str::FromStr;

const RPC_URL: &str = "https://api.devnet.solana.com";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("You've generated a new Solana wallet: {}\n", kp.pubkey());
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as a base58 string:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file format is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a JSON byte array (e.g. [12,34,...]):");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        println!("Your Base58-encoded private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    #[test]
    fn claim_airdrop() {
        let keypair = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");
        
        let client = RpcClient::new(RPC_URL);
        
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(sig) => {
                println!("Success! Check your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", sig);
            }
            Err(err) => {
                println!("Airdrop failed: {}", err);
            }
        }
    }

    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let pubkey = keypair.pubkey();
        let message_bytes = b"I verify my Solana Keypair!";
        let sig = keypair.sign_message(message_bytes);
        let sig_hashed = hash(sig.as_ref());

        match sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes()) {
            true => println!("Signature verified"),
            false => println!("Verification failed"),
        }

        let to_pubkey = Pubkey::from_str("GffKpKRd1ts7kGoEtkJDK84bkufz9itmQfKkcEKLsTCT").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);

        let balance = rpc_client.get_balance(&keypair.pubkey()).unwrap_or(0);
        println!("Current balance: {} lamports", balance);

        let transfer_amount = 100_000_000u64; // 0.1 SOL
        let min_balance = transfer_amount + 5_000; // 0.1 SOL + 0.000005 SOL for fees

        if balance < min_balance {
            println!("Insufficient balance for transfer. Need at least {} lamports, have {}", min_balance, balance);
            println!("Please run 'cargo test claim_airdrop' first or fund your account");
            return;
        }

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, transfer_amount)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn empty_wallet() {
        let keypair = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");
        
        let to_pubkey = Pubkey::from_str("GffKpKRd1ts7kGoEtkJDK84bkufz9itmQfKkcEKLsTCT").unwrap();
        
        let rpc_client = RpcClient::new(RPC_URL);
        
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
        
        println!("Current balance: {} lamports", balance);
        
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );
        
        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");
        
        println!("Estimated fee: {} lamports", fee);
        println!("Transferring: {} lamports", balance - fee);
        
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );
        
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send final transaction");
        
        println!(
            "Success! Entire balance transferred: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn initialize_turbin3() {
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let rpc_client = RpcClient::new(RPC_URL);

        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);

        let github_username = "yashvikram30"; // TODO: Replace with your GitHub username
        let mut data = vec![175, 175, 109, 31, 13, 152, 155, 237]; // discriminator for initialize
        data.extend_from_slice(&(github_username.len() as u32).to_le_bytes()); // string length prefix
        data.extend_from_slice(github_username.as_bytes()); // github username bytes

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),      // user signer
            AccountMeta::new(prereq_pda, false),          // PDA account
            AccountMeta::new_readonly(system_program, false), // system program
        ];

        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn update_github() {
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let rpc_client = RpcClient::new(RPC_URL);

        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);

        let github_username = "yashvikram30"; // TODO: Replace with your GitHub username
        let mut data = vec![219, 200, 88, 176, 158, 63, 253, 127]; // discriminator for update
        data.extend_from_slice(&(github_username.len() as u32).to_le_bytes()); // string length prefix
        data.extend_from_slice(github_username.as_bytes()); // github username bytes

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),      // user signer
            AccountMeta::new(prereq_pda, false),          // PDA account
            AccountMeta::new_readonly(system_program, false), // system program
        ];

        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn check_balance() {
        let keypair = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let rpc_client = RpcClient::new(RPC_URL);

        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        println!("Current balance: {} lamports", balance);
        println!("Current balance: {} SOL", balance as f64 / 1_000_000_000.0);

        let min_rent = rpc_client.get_minimum_balance_for_rent_exemption(0).unwrap();
        println!("Minimum rent exemption: {} lamports", min_rent);
    }

    #[test]
    fn submit_turbin3() {
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let rpc_client = RpcClient::new(RPC_URL);

        let mint = Keypair::new();
        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();
        let mpl_core_program = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);
        
        let authority_seeds: &[&[u8]] = &[b"collection", collection.as_ref()];
        let (authority, _auth_bump) = Pubkey::find_program_address(authority_seeds, &turbin3_prereq_program);
        
        let data = vec![77, 124, 82, 163, 21, 133, 181, 206];
        
        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),      // user signer
            AccountMeta::new(prereq_pda, false),          // PDA account
            AccountMeta::new(mint.pubkey(), true),        // mint keypair
            AccountMeta::new(collection, false),          // collection (writable, NOT signer)
            AccountMeta::new_readonly(authority, false),  // authority (PDA)
            AccountMeta::new_readonly(mpl_core_program, false), // mpl core program
            AccountMeta::new_readonly(system_program, false),   // system program
        ];
        
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };
        
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer, &mint],
            blockhash,
        );
        
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn close_wrong_nft() {
        let dev_keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find dev wallet file");
        
        let rpc_client = RpcClient::new(RPC_URL);
        
        let nft_address = Pubkey::from_str("FdMd2HTa5B5KcfotVWZMGxvmBr3YBmAGWQBSYPcNZHW").unwrap();
        
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        let close_instruction = Instruction {
            program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(), // System program
            accounts: vec![
                AccountMeta::new(nft_address, true),           // account to close
                AccountMeta::new(dev_keypair.pubkey(), true),  // destination for reclaimed rent
                AccountMeta::new_readonly(dev_keypair.pubkey(), false), // owner (signer)
            ],
            data: vec![], // Close account instruction
        };
        
        let transaction = Transaction::new_signed_with_payer(
            &[close_instruction],
            Some(&dev_keypair.pubkey()),
            &[&dev_keypair],
            recent_blockhash,
        );
        
        match rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                println!("Successfully closed NFT account! TX: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
                println!("This effectively removes the NFT from circulation and reclaims rent.");
            }
            Err(err) => {
                println!("Failed to close NFT: {}", err);
            }
        }
    }

    #[test]
    fn submit_ts_turbin3() {
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

        let rpc_client = RpcClient::new(RPC_URL);

        let mint = Keypair::new();
        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();
        let mpl_core_program = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);

        let authority_seeds: &[&[u8]] = &[b"collection", collection.as_ref()];
        let (authority, _auth_bump) = Pubkey::find_program_address(authority_seeds, &turbin3_prereq_program);

        let data = vec![137, 241, 199, 223, 125, 33, 85, 217];

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),      // user signer
            AccountMeta::new(prereq_pda, false),          // PDA account
            AccountMeta::new(mint.pubkey(), true),        // mint keypair
            AccountMeta::new(collection, false),          // collection (writable, NOT signer)
            AccountMeta::new_readonly(authority, false),  // authority (PDA)
            AccountMeta::new_readonly(mpl_core_program, false), // mpl core program
            AccountMeta::new_readonly(system_program, false),   // system program
        ];

        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer, &mint],
            blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}