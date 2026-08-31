//! Rust 错误处理模式实战：thiserror（库层）+ anyhow（应用层）。
//!
//! 分层原则：
//! - 库代码用 `thiserror` 定义有语义的错误类型，让调用方能匹配处理
//! - 应用代码用 `anyhow` 统一包装，附加上下文，简化 `?` 链式传播

use thiserror::Error;

// ==================== 库层：语义化错误类型 ====================

/// 账户领域的错误：每个变体都有明确的业务含义
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("账户 '{0}' 不存在")]
    NotFound(String),

    #[error("余额不足: 需要 {required}，实际 {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("金额无效: {0}")]
    InvalidAmount(u64),
}

/// 模拟的账户服务（库层代码）
pub struct AccountService;

impl AccountService {
    /// 取款：返回具体的领域错误，供调用方精确匹配
    pub fn withdraw(&self, name: &str, amount: u64) -> Result<u64, AccountError> {
        let balance = self.balance_of(name)?;
        if amount == 0 {
            return Err(AccountError::InvalidAmount(amount));
        }
        if amount > balance {
            return Err(AccountError::InsufficientFunds {
                required: amount,
                available: balance,
            });
        }
        Ok(balance - amount)
    }

    fn balance_of(&self, name: &str) -> Result<u64, AccountError> {
        match name {
            "alice" => Ok(1_000),
            "bob" => Ok(50),
            _ => Err(AccountError::NotFound(name.to_string())),
        }
    }
}

// ==================== 应用层：anyhow 上下文包装 ====================

fn main() {
    // `{:#}` 打印完整错误链（所有 context 一层层展开）
    if let Err(e) = run() {
        eprintln!("程序失败: {:#}", e);
        std::process::exit(1);
    }
    println!("\n全部场景演示完成");
}

fn run() -> anyhow::Result<()> {
    let svc = AccountService;

    // --- 1. 正常路径：? 自动把 AccountError 转换为 anyhow::Error ---
    let remaining = svc.withdraw("alice", 200)?;
    println!("alice 取款 200 成功，剩余余额: {}", remaining);

    // --- 2. 业务错误精确匹配：matches! / match ---
    match svc.withdraw("bob", 100) {
        Err(AccountError::InsufficientFunds { required, available }) => {
            println!("bob 余额不足（需要 {}，只有 {}），按业务规则跳过", required, available);
        }
        Err(e) => return Err(e.into()), // 其他错误上抛
        Ok(_) => unreachable!(),
    }

    // --- 3. anyhow 上下文：为错误附加"在哪一步失败"的信息 ---
    use anyhow::Context;
    process_withdrawal(&svc, "alice", 300)
        .context("处理每日例行扣费时失败")?;

    // --- 4. downcast：anyhow::Error 里找回原始类型错误 ---
    // with_context 的懒求值版本：只在出错时执行闭包
    let result: anyhow::Result<()> = (|| {
        svc.withdraw("ghost", 10)?;
        Ok(())
    })();
    if let Err(e) = result {
        if let Some(account_err) = e.downcast_ref::<AccountError>() {
            println!("通过 downcast 识别出领域错误: {}", account_err);
        } else {
            println!("未识别的错误: {:?}", e);
        }
    }

    // --- 5. 最终上抛：这一行会触发 main 中的错误打印，展示完整错误链 ---
    process_withdrawal(&svc, "ghost", 10)
        .context("结算月度账单时失败")?;

    Ok(())
}

/// 模拟一个多步骤业务流程：每一步用 with_context 标注失败位置
fn process_withdrawal(svc: &AccountService, name: &str, amount: u64) -> anyhow::Result<()> {
    let balance = svc
        .withdraw(name, amount)
        .map_err(anyhow::Error::new)
        .map_err(|e| e.context(format!("账户 {} 取款 {} 失败", name, amount)))?;
    println!("{} 取款 {} 成功，剩余 {}", name, amount, balance);
    Ok(())
}
