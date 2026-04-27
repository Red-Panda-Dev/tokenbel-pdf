//! Report type enumeration.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy)]
struct VariantDescriptor {
    russian: &'static str,
    snake: &'static str,
    aliases: &'static [&'static str],
}

const VARIANTS: &[(ReportType, VariantDescriptor)] = &[
    (
        ReportType::BalanceSheet,
        VariantDescriptor {
            russian: "Баланс",
            snake: "balance_sheet",
            aliases: &["balance_sheet", "balance"],
        },
    ),
    (
        ReportType::IncomeStatement,
        VariantDescriptor {
            russian: "Отчёт о прибылях и убытках",
            snake: "income_statement",
            aliases: &["income_statement", "income"],
        },
    ),
    (
        ReportType::StatementCashFlow,
        VariantDescriptor {
            russian: "Отчёт о движении денежных средств",
            snake: "statement_cash_flow",
            aliases: &["statement_cash_flow", "cash_flow", "cashflow"],
        },
    ),
    (
        ReportType::StatementEquityChanges,
        VariantDescriptor {
            russian: "Отчёт об изменениях капитала",
            snake: "statement_equity_changes",
            aliases: &["statement_equity_changes", "equity", "capital"],
        },
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    BalanceSheet,
    IncomeStatement,
    StatementCashFlow,
    StatementEquityChanges,
}

impl ReportType {
    fn descriptor(&self) -> &'static VariantDescriptor {
        &VARIANTS
            .iter()
            .find(|(v, _)| *v == *self)
            .expect("every ReportType variant has a descriptor")
            .1
    }

    #[must_use]
    pub fn russian_name(&self) -> &'static str {
        self.descriptor().russian
    }

    pub fn try_from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        for (variant, desc) in VARIANTS {
            if desc.aliases.iter().any(|a| lower.contains(a)) {
                return Some(*variant);
            }
        }
        None
    }
}

impl fmt::Display for ReportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.descriptor().snake)
    }
}

impl std::str::FromStr for ReportType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        for (variant, desc) in VARIANTS {
            if desc.aliases.iter().any(|a| *a == lower) {
                return Ok(*variant);
            }
        }
        Err(format!("Unknown report type: {}", s))
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
