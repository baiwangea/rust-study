//! 测试实践示例：单元测试、集成测试与属性测试。
//!
//! 测试组织方式：
//! - 单元测试：与被测代码同文件，`#[cfg(test)] mod tests`，可访问私有函数
//! - 集成测试：`tests/` 目录，作为外部使用者只能访问公开 API
//! - 属性测试：`proptest` 生成大量随机输入验证不变式（见本文件底部）
//!
//! 运行：`cargo test -p testing-examples`

use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum CalcError {
    #[error("除数不能为零")]
    DivisionByZero,
}

/// 一个待测的小型计算器
pub struct Calculator;

impl Calculator {
    pub fn add(a: i64, b: i64) -> i64 {
        a + b
    }

    /// 除法：用 Result 表达"除零"这一可预期错误
    pub fn div(a: i64, b: i64) -> Result<i64, CalcError> {
        if b == 0 {
            return Err(CalcError::DivisionByZero);
        }
        Ok(a / b)
    }
}

/// 判断回文（供属性测试使用）
pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars.iter().eq(chars.iter().rev())
}

/// 私有函数：只有同文件的单元测试能访问，集成测试访问不到。
/// （仅被测试使用时非测试编译会报 dead_code，这里显式允许）
#[allow(dead_code)]
fn internal_helper(x: i64) -> i64 {
    x * 2
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Calculator::add(2, 3), 5);
        assert_eq!(Calculator::add(-1, 1), 0);
    }

    #[test]
    fn test_div_ok() {
        assert_eq!(Calculator::div(10, 2), Ok(5));
    }

    /// 断言错误变体，而不是 unwrap 失败
    #[test]
    fn test_div_by_zero() {
        let err = Calculator::div(10, 0).unwrap_err();
        assert!(matches!(err, CalcError::DivisionByZero));
        assert_eq!(err.to_string(), "除数不能为零");
    }

    /// 预期 panic 的场景用 #[should_panic]
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_out_of_bounds_panics() {
        let v = vec![1, 2, 3];
        let _ = v[10];
    }

    #[test]
    fn test_internal_helper() {
        // 单元测试可以访问私有函数
        assert_eq!(internal_helper(21), 42);
    }
}

// ==================== 属性测试（proptest） ====================

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// 不变式：加法满足交换律
        #[test]
        fn add_is_commutative(a in -100000i64..100000, b in -100000i64..100000) {
            prop_assert_eq!(Calculator::add(a, b), Calculator::add(b, a));
        }

        /// 不变式：任意字符串拼接自己的反转，结果一定是回文
        #[test]
        fn doubled_with_reverse_is_palindrome(s in "[a-z]{0,20}") {
            let doubled = format!("{}{}", s, s.chars().rev().collect::<String>());
            prop_assert!(is_palindrome(&doubled));
        }
    }
}
