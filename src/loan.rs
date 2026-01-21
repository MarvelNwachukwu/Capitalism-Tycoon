/// Loan type determines repayment structure and interest rate
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoanType {
    /// Manual payments, highest interest (base + 2%)
    Flexible,
    /// Auto-deduct 2% daily, medium interest (base + 1%)
    LineOfCredit,
    /// Due at end of term, lowest interest (base rate)
    TermLoan,
}

impl LoanType {
    /// Returns the interest rate modifier added to base rate
    pub fn rate_modifier(&self) -> f64 {
        match self {
            LoanType::Flexible => 0.02,      // +2%
            LoanType::LineOfCredit => 0.01,  // +1%
            LoanType::TermLoan => 0.0,       // Base rate
        }
    }

    /// Returns a description of the loan type
    pub fn description(&self) -> &'static str {
        match self {
            LoanType::Flexible => "Manual payments, pay any amount anytime",
            LoanType::LineOfCredit => "Auto-deduct 2% of balance daily (min $10)",
            LoanType::TermLoan => "Full amount due at end of term",
        }
    }

    /// Returns the display name
    pub fn name(&self) -> &'static str {
        match self {
            LoanType::Flexible => "Flexible Loan",
            LoanType::LineOfCredit => "Line of Credit",
            LoanType::TermLoan => "Term Loan",
        }
    }
}

/// Represents a loan taken by the player
#[derive(Debug, Clone)]
pub struct Loan {
    pub id: u32,
    pub loan_type: LoanType,
    pub principal: f64,           // Original amount borrowed
    pub balance: f64,             // Current amount owed (principal + accrued interest)
    pub interest_rate: f64,       // Annual interest rate (e.g., 0.08 for 8%)
    pub days_remaining: Option<u32>, // For term loans only
    pub daily_payment: f64,       // For line of credit (calculated at creation)
}

impl Loan {
    /// Minimum loan amount
    pub const MIN_LOAN: f64 = 500.0;
    /// Maximum single loan amount
    pub const MAX_LOAN: f64 = 25_000.0;
    /// Maximum total debt across all loans
    pub const MAX_TOTAL_DEBT: f64 = 50_000.0;
    /// Default penalty for term loan default (25%)
    pub const TERM_LOAN_PENALTY: f64 = 0.25;

    /// Creates a new flexible loan (manual payments)
    pub fn new_flexible(id: u32, amount: f64, annual_rate: f64) -> Self {
        Loan {
            id,
            loan_type: LoanType::Flexible,
            principal: amount,
            balance: amount,
            interest_rate: annual_rate,
            days_remaining: None,
            daily_payment: 0.0,
        }
    }

    /// Creates a new line of credit (auto-deduct 2% daily)
    pub fn new_line_of_credit(id: u32, amount: f64, annual_rate: f64) -> Self {
        // Daily payment is 2% of principal or $10, whichever is greater
        let daily_payment = (amount * 0.02).max(10.0);
        Loan {
            id,
            loan_type: LoanType::LineOfCredit,
            principal: amount,
            balance: amount,
            interest_rate: annual_rate,
            days_remaining: None,
            daily_payment,
        }
    }

    /// Creates a new term loan with specified duration
    pub fn new_term_loan(id: u32, amount: f64, annual_rate: f64, days: u32) -> Self {
        Loan {
            id,
            loan_type: LoanType::TermLoan,
            principal: amount,
            balance: amount,
            interest_rate: annual_rate,
            days_remaining: Some(days),
            daily_payment: 0.0,
        }
    }

    /// Returns the daily interest rate
    pub fn daily_rate(&self) -> f64 {
        self.interest_rate / 365.0
    }

    /// Accrue one day's interest on the loan
    pub fn accrue_interest(&mut self) {
        let daily_interest = self.balance * self.daily_rate();
        self.balance += daily_interest;
    }

    /// Make a payment on the loan. Returns the actual amount paid.
    pub fn make_payment(&mut self, amount: f64) -> f64 {
        let actual_payment = amount.min(self.balance);
        self.balance -= actual_payment;
        actual_payment
    }

    /// Check if this term loan is due (days_remaining == 0)
    pub fn is_due(&self) -> bool {
        matches!(self.days_remaining, Some(0))
    }

    /// Check if this term loan is coming due soon (1-3 days remaining)
    pub fn is_due_soon(&self) -> Option<u32> {
        match self.days_remaining {
            Some(days) if days > 0 && days <= 3 => Some(days),
            _ => None,
        }
    }

    /// Decrement days remaining for term loans
    pub fn decrement_days(&mut self) {
        if let Some(ref mut days) = self.days_remaining {
            if *days > 0 {
                *days -= 1;
            }
        }
    }

    /// Check if loan is fully paid off
    pub fn is_paid_off(&self) -> bool {
        self.balance < 0.01 // Allow for floating point imprecision
    }

    /// Get the required auto-payment amount for line of credit
    /// Returns 2% of current balance or $10, whichever is greater
    pub fn get_auto_payment(&self) -> f64 {
        if self.loan_type == LoanType::LineOfCredit {
            (self.balance * 0.02).max(10.0).min(self.balance)
        } else {
            0.0
        }
    }

    /// Calculate penalty for defaulting on a term loan (25% of balance)
    pub fn default_penalty(&self) -> f64 {
        if self.loan_type == LoanType::TermLoan {
            self.balance * Self::TERM_LOAN_PENALTY
        } else {
            0.0
        }
    }

    /// Format interest rate for display (annual percentage)
    pub fn display_rate(&self) -> String {
        format!("{:.1}%", self.interest_rate * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flexible_loan_creation() {
        let loan = Loan::new_flexible(1, 1000.0, 0.08);
        assert_eq!(loan.loan_type, LoanType::Flexible);
        assert_eq!(loan.principal, 1000.0);
        assert_eq!(loan.balance, 1000.0);
        assert_eq!(loan.interest_rate, 0.08);
        assert!(loan.days_remaining.is_none());
    }

    #[test]
    fn test_line_of_credit_creation() {
        let loan = Loan::new_line_of_credit(1, 1000.0, 0.07);
        assert_eq!(loan.loan_type, LoanType::LineOfCredit);
        assert_eq!(loan.daily_payment, 20.0); // 2% of 1000

        // Test minimum payment
        let small_loan = Loan::new_line_of_credit(2, 200.0, 0.07);
        assert_eq!(small_loan.daily_payment, 10.0); // Minimum $10
    }

    #[test]
    fn test_term_loan_creation() {
        let loan = Loan::new_term_loan(1, 5000.0, 0.06, 14);
        assert_eq!(loan.loan_type, LoanType::TermLoan);
        assert_eq!(loan.days_remaining, Some(14));
    }

    #[test]
    fn test_interest_accrual() {
        let mut loan = Loan::new_flexible(1, 1000.0, 0.0365); // ~0.01% daily
        loan.accrue_interest();
        assert!(loan.balance > 1000.0);
        assert!((loan.balance - 1000.10).abs() < 0.01); // ~$0.10 daily interest
    }

    #[test]
    fn test_payment() {
        let mut loan = Loan::new_flexible(1, 1000.0, 0.08);
        let paid = loan.make_payment(300.0);
        assert_eq!(paid, 300.0);
        assert_eq!(loan.balance, 700.0);

        // Test overpayment
        let paid = loan.make_payment(1000.0);
        assert_eq!(paid, 700.0);
        assert!(loan.is_paid_off());
    }

    #[test]
    fn test_term_loan_due() {
        let mut loan = Loan::new_term_loan(1, 1000.0, 0.06, 2);
        assert!(!loan.is_due());
        assert_eq!(loan.is_due_soon(), Some(2));

        loan.decrement_days();
        assert!(!loan.is_due());
        assert_eq!(loan.is_due_soon(), Some(1));

        loan.decrement_days();
        assert!(loan.is_due());
    }

    #[test]
    fn test_auto_payment() {
        let loan = Loan::new_line_of_credit(1, 1000.0, 0.07);
        assert_eq!(loan.get_auto_payment(), 20.0); // 2% of 1000

        let small_loan = Loan::new_line_of_credit(2, 300.0, 0.07);
        assert_eq!(small_loan.get_auto_payment(), 10.0); // Minimum $10
    }
}
