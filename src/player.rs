use crate::factory::Factory;
use crate::loan::Loan;
use crate::store::Store;

/// Represents the player in the game
#[derive(Debug)]
pub struct Player {
    pub cash: f64,
    pub stores: Vec<Store>,
    pub factories: Vec<Factory>,
    pub loans: Vec<Loan>,
    next_store_id: u32,
    next_factory_id: u32,
    next_loan_id: u32,
}

impl Player {
    /// Creates a new player with starting cash and one store
    pub fn new(starting_cash: f64, store_name: &str) -> Self {
        Player {
            cash: starting_cash,
            stores: vec![Store::new(1, store_name)],
            factories: Vec::new(),
            loans: Vec::new(),
            next_store_id: 2,
            next_factory_id: 1,
            next_loan_id: 1,
        }
    }

    /// Gets a reference to a store by index
    pub fn store_at(&self, index: usize) -> &Store {
        &self.stores[index]
    }

    /// Gets a mutable reference to a store by index
    pub fn store_at_mut(&mut self, index: usize) -> &mut Store {
        &mut self.stores[index]
    }

    /// Gets a reference to the player's first store (for backwards compatibility)
    pub fn store(&self) -> &Store {
        &self.stores[0]
    }

    /// Gets a mutable reference to the player's first store (for backwards compatibility)
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.stores[0]
    }

    /// Adds a new store to the player's portfolio
    pub fn add_store(&mut self, name: &str) {
        let store = Store::new(self.next_store_id, name);
        self.stores.push(store);
        self.next_store_id += 1;
    }

    /// Gets a reference to a factory by index
    pub fn factory_at(&self, index: usize) -> &Factory {
        &self.factories[index]
    }

    /// Gets a mutable reference to a factory by index
    pub fn factory_at_mut(&mut self, index: usize) -> &mut Factory {
        &mut self.factories[index]
    }

    /// Adds a new factory to the player's portfolio
    pub fn add_factory(&mut self, name: &str) {
        let factory = Factory::new(self.next_factory_id, name);
        self.factories.push(factory);
        self.next_factory_id += 1;
    }

    /// Spends money if the player has enough
    pub fn spend(&mut self, amount: f64) -> bool {
        if self.cash >= amount {
            self.cash -= amount;
            true
        } else {
            false
        }
    }

    /// Adds money to the player's cash
    pub fn earn(&mut self, amount: f64) {
        self.cash += amount;
    }

    /// Returns the player's total net worth (cash + inventory value - debt)
    pub fn net_worth(&self) -> f64 {
        let inventory_value: f64 = self.stores.iter().map(|s| s.total_inventory_value()).sum();
        self.cash + inventory_value - self.total_debt()
    }

    /// Returns the total daily expenses across all stores and factories
    pub fn total_daily_expenses(&self) -> f64 {
        let store_expenses: f64 = self.stores.iter().map(|s| s.daily_expenses()).sum();
        let factory_expenses: f64 = self.factories.iter().map(|f| f.daily_expenses()).sum();
        store_expenses + factory_expenses
    }

    /// Returns the total debt across all loans
    pub fn total_debt(&self) -> f64 {
        self.loans.iter().map(|l| l.balance).sum()
    }

    /// Adds a new loan to the player
    pub fn add_loan(&mut self, mut loan: Loan) {
        loan.id = self.next_loan_id;
        self.next_loan_id += 1;
        self.cash += loan.principal; // Receive the loan amount
        self.loans.push(loan);
    }

    /// Returns whether the player can borrow the specified amount
    pub fn can_borrow(&self, amount: f64) -> bool {
        let new_total = self.total_debt() + amount;
        new_total <= Loan::MAX_TOTAL_DEBT
    }

    /// Returns the maximum amount the player can still borrow
    pub fn max_borrowable(&self) -> f64 {
        (Loan::MAX_TOTAL_DEBT - self.total_debt()).max(0.0)
    }

    /// Gets a reference to a loan by ID
    pub fn get_loan(&self, loan_id: u32) -> Option<&Loan> {
        self.loans.iter().find(|l| l.id == loan_id)
    }

    /// Gets a mutable reference to a loan by ID
    pub fn get_loan_mut(&mut self, loan_id: u32) -> Option<&mut Loan> {
        self.loans.iter_mut().find(|l| l.id == loan_id)
    }

    /// Makes a payment on a specific loan. Returns the actual amount paid.
    pub fn make_loan_payment(&mut self, loan_id: u32, amount: f64) -> Option<f64> {
        let payment_amount = amount.min(self.cash);
        if let Some(loan) = self.get_loan_mut(loan_id) {
            let actual_paid = loan.make_payment(payment_amount);
            self.cash -= actual_paid;
            Some(actual_paid)
        } else {
            None
        }
    }

    /// Removes all paid-off loans
    pub fn cleanup_loans(&mut self) {
        self.loans.retain(|l| !l.is_paid_off());
    }

    /// Returns loans that are coming due soon (1-3 days)
    pub fn loans_due_soon(&self) -> Vec<(u32, u32)> {
        self.loans
            .iter()
            .filter_map(|l| l.is_due_soon().map(|days| (l.id, days)))
            .collect()
    }

    /// Returns the next loan ID without incrementing
    pub fn peek_next_loan_id(&self) -> u32 {
        self.next_loan_id
    }
}
