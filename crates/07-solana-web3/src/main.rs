use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 连接到 Solana Devnet
    let rpc_url = String::from("https://api.devnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    println!("成功连接到 Solana Devnet");

    // 2. 创建一个新的密钥对 (账户)
    let new_account = Keypair::new();
    let pubkey = new_account.pubkey();
    println!("创建新账户，公钥: {}", pubkey);

    // 3. 为新账户请求空投 (Airdrop)
    println!("正在为新账户请求 1 SOL 的空投...");
    let signature = client.request_airdrop(&pubkey, 1_000_000_000)?; // 1 SOL = 1,000,000,000 Lamports
    
    // 等待空投交易确认
    println!("等待交易确认...");
    loop {
        let status = client.get_signature_status(&signature)?;
        match status {
            Some(Ok(_)) => {
                println!("空投成功！交易签名: {}", signature);
                break;
            }
            Some(Err(err)) => {
                println!("交易失败: {:?}", err);
                break;
            }
            None => {
                // 等待一会再查询
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    // 4. 查询账户的余额
    let balance = client.get_balance(&pubkey)?;
    println!("账户 {} 的当前余额是: {} Lamports ({} SOL)", pubkey, balance, balance as f64 / 1_000_000_000.0);

    println!("\nSolana 交互示例执行完毕。");

    Ok(())
}
