//! 集成测试：位于 `tests/` 目录，作为库的外部使用者，
//! 只能访问公开 API（例如这里访问不到 `internal_helper`）。

use testing_examples::{CalcError, Calculator, is_palindrome};

#[test]
fn test_calculator_from_outside() {
    assert_eq!(Calculator::add(1, 2), 3);
    assert!(matches!(
        Calculator::div(1, 0),
        Err(CalcError::DivisionByZero)
    ));
}

#[test]
fn test_palindrome_from_outside() {
    assert!(is_palindrome("racecar"));
    assert!(is_palindrome("上海自来水来自海上"));
    assert!(!is_palindrome("rust"));
    assert!(is_palindrome("")); // 空串视为回文
}
