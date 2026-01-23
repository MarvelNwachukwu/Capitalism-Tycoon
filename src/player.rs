use std::collections::HashMap;
use crate::factory::Factory;
use crate::loan::Loan;
use crate::stock::StockHolding;
use crate::store::Store;

/// Represents the player in the game
#[derive(Debug)]
pub struct Player {
    pub cash: f64,
    pub stores: Vec<Store>,
    pub factories: Vec<Factory>,
    pub loans: Vec<Loan>,
    /// Stock portfolio: stock_id -> holding
    pub portfolio: HashMap<u32, StockHolding>,
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
            portfolio: HashMap::new(),
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

    /// Returns the player's total net worth (cash + inventory value + portfolio cost basis - debt)
    /// Note: For accurate net worth with current prices, use net_worth_with_stocks
    pub fn net_worth(&self) -> f64 {
        let inventory_value: f64 = self.stores.iter().map(|s| s.total_inventory_value()).sum();
        let portfolio_cost: f64 = self.portfolio.values()
            .map(|h| h.avg_purchase_price * h.shares as f64)
            .sum();
        self.cash + inventory_value + portfolio_cost - self.total_debt()
    }

    /// Returns net worth including current stock market values
    pub fn net_worth_with_stocks(&self, stock_prices: &HashMap<u32, f64>) -> f64 {
        let inventory_value: f64 = self.stores.iter().map(|s| s.total_inventory_value()).sum();
        let portfolio_value = self.portfolio_value(stock_prices);
        self.cash + inventory_value + portfolio_value - self.total_debt()
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

    // ==================== STOCK PORTFOLIO METHODS ====================

    /// Buys shares of a stock
    pub fn buy_stock(&mut self, stock_id: u32, shares: u32, price: f64) -> Result<(), String> {
        let total_cost = price * shares as f64;
        if total_cost > self.cash {
            return Err(format!(
                "Not enough cash! Need ${:.2}, have ${:.2}",
                total_cost, self.cash
            ));
        }

        self.cash -= total_cost;

        if let Some(holding) = self.portfolio.get_mut(&stock_id) {
            holding.add_shares(shares, price);
        } else {
            self.portfolio.insert(stock_id, StockHolding::new(stock_id, shares, price));
        }

        Ok(())
    }

    /// Sells shares of a stock
    pub fn sell_stock(&mut self, stock_id: u32, shares: u32, price: f64) -> Result<f64, String> {
        let holding = self.portfolio.get_mut(&stock_id)
            .ok_or("You don't own this stock")?;

        if shares > holding.shares {
            return Err(format!(
                "Not enough shares! You have {}, trying to sell {}",
                holding.shares, shares
            ));
        }

        holding.remove_shares(shares);
        let proceeds = price * shares as f64;
        self.cash += proceeds;

        // Remove holding if no shares left
        if holding.shares == 0 {
            self.portfolio.remove(&stock_id);
        }

        Ok(proceeds)
    }

    /// Gets the holding for a specific stock
    pub fn get_holding(&self, stock_id: u32) -> Option<&StockHolding> {
        self.portfolio.get(&stock_id)
    }

    /// Gets mutable holding for a specific stock
    pub fn get_holding_mut(&mut self, stock_id: u32) -> Option<&mut StockHolding> {
        self.portfolio.get_mut(&stock_id)
    }

    /// Returns total portfolio value at given stock prices
    pub fn portfolio_value(&self, stock_prices: &HashMap<u32, f64>) -> f64 {
        self.portfolio.iter().map(|(stock_id, holding)| {
            let price = stock_prices.get(stock_id).unwrap_or(&0.0);
            holding.current_value(*price)
        }).sum()
    }

    /// Returns total unrealized gain/loss
    pub fn portfolio_gain_loss(&self, stock_prices: &HashMap<u32, f64>) -> f64 {
        self.portfolio.iter().map(|(stock_id, holding)| {
            let price = stock_prices.get(stock_id).unwrap_or(&0.0);
            holding.gain_loss(*price)
        }).sum()
    }

    /// Returns total dividends earned across all holdings
    pub fn total_dividends_earned(&self) -> f64 {
        self.portfolio.values().map(|h| h.total_dividends_earned).sum()
    }
}
