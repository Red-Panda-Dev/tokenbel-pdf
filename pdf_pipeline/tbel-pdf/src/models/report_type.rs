//! Report type enumeration.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported financial report types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Balance sheet (Баланс).
    BalanceSheet,
    /// Income statement (Отчёт о прибылях и убытках).
    IncomeStatement,
    /// Statement of cash flows (Отчёт о движении денежных средств).
    StatementCashFlow,
    /// Statement of changes in equity (Отчёт об изменениях капитала).
    StatementEquityChanges,
}

impl ReportType {
    /// Get the Russian name of the report type.
    #[must_use]
    pub fn russian_name(&self) -> &'static str {
        match self {
            Self::BalanceSheet => "Баланс",
            Self::IncomeStatement => "Отчёт о прибылях и убытках",
            Self::StatementCashFlow => "Отчёт о движении денежных средств",
            Self::StatementEquityChanges => "Отчёт об изменениях капитала",
        }
    }

    /// Try to infer report type from a filename or URL.
    pub fn try_from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        if lower.contains("balance_sheet") || lower.contains("balance") {
            Some(Self::BalanceSheet)
        } else if lower.contains("income_statement") || lower.contains("income") {
            Some(Self::IncomeStatement)
        } else if lower.contains("cash_flow") || lower.contains("cashflow") {
            Some(Self::StatementCashFlow)
        } else if lower.contains("equity") || lower.contains("capital") {
            Some(Self::StatementEquityChanges)
        } else {
            None
        }
    }
}

impl fmt::Display for ReportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BalanceSheet => write!(f, "balance_sheet"),
            Self::IncomeStatement => write!(f, "income_statement"),
            Self::StatementCashFlow => write!(f, "statement_cash_flow"),
            Self::StatementEquityChanges => write!(f, "statement_equity_changes"),
        }
    }
}

impl std::str::FromStr for ReportType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "balance_sheet" | "balance" => Ok(Self::BalanceSheet),
            "income_statement" | "income" => Ok(Self::IncomeStatement),
            "statement_cash_flow" | "cash_flow" | "cashflow" => Ok(Self::StatementCashFlow),
            "statement_equity_changes" | "equity" | "capital" => Ok(Self::StatementEquityChanges),
            _ => Err(format!("Unknown report type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_display() {
        assert_eq!(ReportType::BalanceSheet.to_string(), "balance_sheet");
        assert_eq!(ReportType::IncomeStatement.to_string(), "income_statement");
        assert_eq!(
            ReportType::StatementCashFlow.to_string(),
            "statement_cash_flow"
        );
    }

    #[test]
    fn test_report_type_from_str() {
        assert!(matches!(
            "balance_sheet".parse(),
            Ok(ReportType::BalanceSheet)
        ));
        assert!(matches!(
            "income_statement".parse(),
            Ok(ReportType::IncomeStatement)
        ));
        assert!(matches!(
            "cash_flow".parse(),
            Ok(ReportType::StatementCashFlow)
        ));
        assert!("unknown".parse::<ReportType>().is_err());
    }

    #[test]
    fn test_report_type_try_from_filename() {
        assert_eq!(
            ReportType::try_from_filename("2024_balance_sheet.pdf"),
            Some(ReportType::BalanceSheet)
        );
        assert_eq!(
            ReportType::try_from_filename("income_statement_2024.pdf"),
            Some(ReportType::IncomeStatement)
        );
        assert_eq!(
            ReportType::try_from_filename("cashflow_report.pdf"),
            Some(ReportType::StatementCashFlow)
        );
        assert_eq!(ReportType::try_from_filename("unknown.pdf"), None);
    }

    #[test]
    fn test_report_type_russian_name() {
        assert_eq!(ReportType::BalanceSheet.russian_name(), "Баланс");
        assert_eq!(
            ReportType::IncomeStatement.russian_name(),
            "Отчёт о прибылях и убытках"
        );
    }
}
