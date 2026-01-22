use crate::competitor::CompetitiveMarket;
use crate::economy::{EconomicState, Market};
use crate::factory::ProductionResult;
use crate::loan::{Loan, LoanType};
use crate::player::Player;
use crate::product::Product;
use crate::recipe::Recipe;

/// Represents the complete game state
pub struct GameState {
    pub day: u32,
    pub player: Player,
    pub market: Market,
    pub competitive_market: CompetitiveMarket,
    pub products: Vec<Product>,
    pub recipes: Vec<Recipe>,
    pub current_store: usize,
    pub current_factory: Option<usize>,
    pub is_bankrupt: bool,
}

/// Result of simulating a day's sales
#[derive(Debug)]
pub struct DayResult {
    pub total_revenue: f64,
    pub total_items_sold: u32,
    pub sales_by_product: Vec<(String, u32, f64)>, // (name, quantity, revenue)
    pub total_expenses: f64,
    pub expenses_by_store: Vec<(String, f64, f64)>, // (store_name, rent, salaries)
    pub expenses_by_factory: Vec<(String, f64, f64)>, // (factory_name, rent, salaries)
    pub production_completed: Vec<ProductionResult>,
    pub net_profit: f64,
    // Economic state fields
    pub economic_state: EconomicState,
    pub economic_change: Option<String>,       // "Economy improved to Growth!"
    // Loan fields
    pub loan_interest_accrued: f64,            // Total interest accrued today
    pub loan_payments: Vec<(u32, f64)>,        // (loan_id, amount_paid) - auto-payments
    pub loans_due: Vec<(u32, f64)>,            // Term loans that came due (id, amount)
    pub loans_due_soon: Vec<(u32, u32, f64)>,  // Warnings: (loan_id, days_remaining, balance)
    pub term_loan_penalties: f64,              // Penalties for defaulted term loans
    // Supply chain auto-transfers: (factory_name, store_name, product_name, quantity)
    pub auto_transfers: Vec<(String, String, String, u32)>,
    // Competitor events
    pub competitor_events: Vec<String>,
    pub player_market_share: f64,
}

impl GameState {
    /// Creates a new game with default settings
    pub fn new() -> Self {
        let products = Product::default_products();
        let market = Market::new(&products);
        let player = Player::new(1000.0, "My First Store");
        let recipes = Recipe::default_recipes();
        let competitive_market = CompetitiveMarket::new();

        GameState {
            day: 1,
            player,
            market,
            competitive_market,
            products,
            recipes,
            current_store: 0,
            current_factory: None,
            is_bankrupt: false,
        }
    }

    /// Gets the current store reference
    pub fn current_store(&self) -> &crate::store::Store {
        self.player.store_at(self.current_store)
    }

    /// Gets the current store mutable reference
    pub fn current_store_mut(&mut self) -> &mut crate::store::Store {
        self.player.store_at_mut(self.current_store)
    }

    /// Switches to a different store by index
    pub fn switch_store(&mut self, index: usize) -> Result<(), String> {
        if index >= self.player.stores.len() {
            return Err("Invalid store index".to_string());
        }
        self.current_store = index;
        Ok(())
    }

    /// Buys a new store
    /// Returns competitor reactions to the expansion
    pub fn buy_new_store(&mut self, name: &str) -> Result<Vec<String>, String> {
        const NEW_STORE_COST: f64 = 5000.0;

        if self.player.cash < NEW_STORE_COST {
            return Err(format!(
                "Not enough cash! Need ${:.2}, have ${:.2}",
                NEW_STORE_COST, self.player.cash
            ));
        }

        self.player.spend(NEW_STORE_COST);
        self.player.add_store(name);

        // Notify competitors and get their reactions
        let reactions = self.competitive_market.notify_player_expansion();
        Ok(reactions)
    }

    // ==================== FACTORY METHODS ====================

    /// Gets the current factory reference (if any)
    pub fn current_factory(&self) -> Option<&crate::factory::Factory> {
        self.current_factory.map(|idx| self.player.factory_at(idx))
    }

    /// Gets the current factory mutable reference (if any)
    pub fn current_factory_mut(&mut self) -> Option<&mut crate::factory::Factory> {
        self.current_factory
            .map(|idx| self.player.factory_at_mut(idx))
    }

    /// Switches to a different factory by index
    pub fn switch_factory(&mut self, index: usize) -> Result<(), String> {
        if index >= self.player.factories.len() {
            return Err("Invalid factory index".to_string());
        }
        self.current_factory = Some(index);
        Ok(())
    }

    /// Buys a new factory
    pub fn buy_new_factory(&mut self, name: &str) -> Result<(), String> {
        const NEW_FACTORY_COST: f64 = 10000.0;

        if self.player.cash < NEW_FACTORY_COST {
            return Err(format!(
                "Not enough cash! Need ${:.2}, have ${:.2}",
                NEW_FACTORY_COST, self.player.cash
            ));
        }

        self.player.spend(NEW_FACTORY_COST);
        self.player.add_factory(name);

        // Auto-select the new factory if it's the first one
        if self.current_factory.is_none() {
            self.current_factory = Some(0);
        }

        Ok(())
    }

    /// Gets a recipe by ID
    pub fn get_recipe(&self, recipe_id: u32) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == recipe_id)
    }

    /// Buys raw materials for the current factory
    pub fn buy_raw_materials(&mut self, product_id: u32, quantity: u32) -> Result<f64, String> {
        // Verify we have a factory selected
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        // Verify product is a raw material
        let product = self
            .get_product(product_id)
            .ok_or("Product not found")?;

        if !product.product_type.is_raw_material() {
            return Err("This product is not a raw material".to_string());
        }

        let wholesale_price = self
            .market
            .get_wholesale_price(product_id)
            .ok_or("Wholesale price not found")?;

        let total_cost = wholesale_price * quantity as f64;

        if !self.player.spend(total_cost) {
            return Err(format!(
                "Not enough cash! Need ${:.2}, have ${:.2}",
                total_cost, self.player.cash
            ));
        }

        self.player.factories[factory_idx].add_raw_material(product_id, quantity);

        Ok(total_cost)
    }

    /// Starts production at the current factory
    pub fn start_production(&mut self, recipe_id: u32) -> Result<(), String> {
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        let recipe = self
            .get_recipe(recipe_id)
            .ok_or("Recipe not found")?
            .clone();

        self.player.factories[factory_idx].start_production(&recipe)
    }

    /// Starts batch production at the current factory
    /// Returns the number of jobs actually started
    pub fn start_production_batch(&mut self, recipe_id: u32, quantity: u32) -> Result<u32, String> {
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        let recipe = self
            .get_recipe(recipe_id)
            .ok_or("Recipe not found")?
            .clone();

        self.player.factories[factory_idx].start_production_batch(&recipe, quantity)
    }

    /// Gets the max producible quantity for a recipe at the current factory
    pub fn max_producible(&self, recipe_id: u32) -> Option<u32> {
        let factory = self.current_factory()?;
        let recipe = self.get_recipe(recipe_id)?;
        Some(factory.max_producible(recipe))
    }

    /// Transfers finished goods from factory to store
    /// Requires the factory to be connected to the store (supply chain)
    pub fn transfer_to_store(
        &mut self,
        product_id: u32,
        quantity: u32,
        store_idx: usize,
    ) -> Result<u32, String> {
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        if store_idx >= self.player.stores.len() {
            return Err("Invalid store index".to_string());
        }

        // Check supply chain connection
        let store_id = self.player.stores[store_idx].id;
        if !self.player.factories[factory_idx].is_connected_to(store_id) {
            return Err(format!(
                "Factory is not connected to {}. Set up supply chain first!",
                self.player.stores[store_idx].name
            ));
        }

        // Get product info for retail price
        let product = self
            .get_product(product_id)
            .ok_or("Product not found")?;

        let retail_price = Market::suggest_retail_price(product.base_price, 50.0);

        // Take from factory
        let actual_quantity = self.player.factories[factory_idx]
            .take_finished_goods(product_id, quantity)?;

        // Add to store
        self.player.stores[store_idx].add_inventory(product_id, actual_quantity, retail_price);

        Ok(actual_quantity)
    }

    // ==================== SUPPLY CHAIN METHODS ====================

    /// Connects the current factory to a store
    pub fn connect_factory_to_store(&mut self, store_idx: usize) -> Result<(), String> {
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        if store_idx >= self.player.stores.len() {
            return Err("Invalid store index".to_string());
        }

        let store_id = self.player.stores[store_idx].id;
        self.player.factories[factory_idx].connect_store(store_id);
        Ok(())
    }

    /// Disconnects the current factory from a store
    pub fn disconnect_factory_from_store(&mut self, store_idx: usize) -> Result<(), String> {
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        if store_idx >= self.player.stores.len() {
            return Err("Invalid store index".to_string());
        }

        let store_id = self.player.stores[store_idx].id;
        self.player.factories[factory_idx].disconnect_store(store_id);
        Ok(())
    }

    /// Toggles auto-transfer for the current factory
    pub fn toggle_factory_auto_transfer(&mut self) -> Result<bool, String> {
        let factory_idx = self
            .current_factory
            .ok_or("No factory selected")?;

        self.player.factories[factory_idx].toggle_auto_transfer();
        Ok(self.player.factories[factory_idx].auto_transfer)
    }

    /// Gets store index by store ID
    pub fn get_store_index_by_id(&self, store_id: u32) -> Option<usize> {
        self.player.stores.iter().position(|s| s.id == store_id)
    }

    /// Gets store name by ID
    pub fn get_store_name_by_id(&self, store_id: u32) -> Option<&str> {
        self.player.stores.iter()
            .find(|s| s.id == store_id)
            .map(|s| s.name.as_str())
    }

    /// Calculates total daily expenses across all stores and factories
    pub fn total_daily_expenses(&self) -> f64 {
        self.player.total_daily_expenses()
    }

    /// Gets a product by ID
    pub fn get_product(&self, product_id: u32) -> Option<&Product> {
        self.products.iter().find(|p| p.id == product_id)
    }

    /// Buys inventory from the wholesale market for the current store
    pub fn buy_inventory(&mut self, product_id: u32, quantity: u32) -> Result<f64, String> {
        // Verify product exists
        if self.get_product(product_id).is_none() {
            return Err("Product not found".to_string());
        }

        let wholesale_price = self
            .market
            .get_wholesale_price(product_id)
            .ok_or("Wholesale price not found")?;

        let total_cost = wholesale_price * quantity as f64;

        if !self.player.spend(total_cost) {
            return Err(format!(
                "Not enough cash! Need ${:.2}, have ${:.2}",
                total_cost, self.player.cash
            ));
        }

        // Add to inventory with default markup of 50%
        let suggested_retail = Market::suggest_retail_price(wholesale_price, 50.0);
        self.current_store_mut()
            .add_inventory(product_id, quantity, suggested_retail);

        Ok(total_cost)
    }

    /// Sets the retail price for a product in the current store
    pub fn set_retail_price(&mut self, product_id: u32, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Price must be positive".to_string());
        }

        if self.current_store_mut().set_price(product_id, price) {
            Ok(())
        } else {
            Err("Product not in inventory".to_string())
        }
    }

    // ==================== LOAN METHODS ====================

    /// Takes out a new flexible loan
    pub fn take_flexible_loan(&mut self, amount: f64) -> Result<u32, String> {
        self.validate_loan_amount(amount)?;

        let rate = self.market.get_loan_rate(&LoanType::Flexible);
        let loan = Loan::new_flexible(0, amount, rate);
        let id = self.player.peek_next_loan_id();
        self.player.add_loan(loan);
        Ok(id)
    }

    /// Takes out a new line of credit
    pub fn take_line_of_credit(&mut self, amount: f64) -> Result<u32, String> {
        self.validate_loan_amount(amount)?;

        let rate = self.market.get_loan_rate(&LoanType::LineOfCredit);
        let loan = Loan::new_line_of_credit(0, amount, rate);
        let id = self.player.peek_next_loan_id();
        self.player.add_loan(loan);
        Ok(id)
    }

    /// Takes out a new term loan with specified duration
    pub fn take_term_loan(&mut self, amount: f64, days: u32) -> Result<u32, String> {
        self.validate_loan_amount(amount)?;

        if !matches!(days, 7 | 14 | 30) {
            return Err("Term loan must be 7, 14, or 30 days".to_string());
        }

        // Apply term discount: -0.5% for 14 days, -1% for 30 days
        let base_rate = self.market.get_loan_rate(&LoanType::TermLoan);
        let rate = match days {
            14 => (base_rate - 0.005).max(0.01),
            30 => (base_rate - 0.01).max(0.01),
            _ => base_rate,
        };

        let loan = Loan::new_term_loan(0, amount, rate, days);
        let id = self.player.peek_next_loan_id();
        self.player.add_loan(loan);
        Ok(id)
    }

    /// Validates loan amount against limits
    fn validate_loan_amount(&self, amount: f64) -> Result<(), String> {
        if amount < Loan::MIN_LOAN {
            return Err(format!("Minimum loan is ${:.2}", Loan::MIN_LOAN));
        }
        if amount > Loan::MAX_LOAN {
            return Err(format!("Maximum single loan is ${:.2}", Loan::MAX_LOAN));
        }
        if !self.player.can_borrow(amount) {
            let max_available = self.player.max_borrowable();
            return Err(format!(
                "Would exceed maximum debt limit of ${:.2}. You can borrow up to ${:.2} more.",
                Loan::MAX_TOTAL_DEBT,
                max_available
            ));
        }
        Ok(())
    }

    /// Makes a manual payment on a loan
    pub fn make_loan_payment(&mut self, loan_id: u32, amount: f64) -> Result<f64, String> {
        if amount <= 0.0 {
            return Err("Payment amount must be positive".to_string());
        }
        if self.player.cash < amount {
            return Err(format!(
                "Not enough cash! Have ${:.2}, trying to pay ${:.2}",
                self.player.cash, amount
            ));
        }

        self.player
            .make_loan_payment(loan_id, amount)
            .ok_or_else(|| "Loan not found".to_string())
    }

    /// Gets the current interest rate for a loan type
    pub fn get_current_loan_rate(&self, loan_type: &LoanType) -> f64 {
        self.market.get_loan_rate(loan_type)
    }

    /// Advances to the next day and simulates sales for ALL stores
    pub fn advance_day(&mut self) -> DayResult {
        // Update economy and get any change message
        let economic_change = self.market.advance_day(self.day);
        let economic_state = self.market.economic_state;

        // Calculate player's average markup for market share calculation
        let player_avg_markup = self.calculate_average_markup();
        let player_store_count = self.player.stores.len() as u32;

        // Update market shares based on player and competitor positions
        self.competitive_market.calculate_market_shares(player_store_count, player_avg_markup);
        let player_market_share = self.competitive_market.player_market_share;
        let customer_multiplier = self.competitive_market.player_customer_multiplier();

        // Process competitor actions
        let competitor_events = self.competitive_market.advance_day(economic_state.sales_multiplier());

        let mut total_revenue = 0.0;
        let mut total_items_sold = 0;
        let mut sales_by_product = Vec::new();
        let mut total_expenses = 0.0;
        let mut expenses_by_store = Vec::new();
        let mut expenses_by_factory = Vec::new();
        let mut production_completed = Vec::new();

        // Loan-related tracking
        let mut loan_interest_accrued = 0.0;
        let mut loan_payments = Vec::new();
        let mut loans_due = Vec::new();
        let mut loans_due_soon = Vec::new();
        let mut term_loan_penalties = 0.0;

        // Process each store
        let store_count = self.player.stores.len();
        for store_idx in 0..store_count {
            // Calculate expenses for this store
            let store = &self.player.stores[store_idx];
            let rent = store.daily_rent;
            let salaries: f64 = store.employees.iter().map(|e| e.salary).sum();
            let store_name = store.name.clone();
            let store_expenses = rent + salaries;
            total_expenses += store_expenses;
            expenses_by_store.push((store_name, rent, salaries));

            // Get customer count with employee bonus and market share multiplier
            let base_customers = self.player.stores[store_idx].effective_customers();
            let customer_count = (base_customers as f64 * customer_multiplier) as u32;

            // Clone inventory keys to avoid borrow issues
            let product_ids: Vec<u32> = self.player.stores[store_idx]
                .inventory
                .keys()
                .copied()
                .collect();

            for product_id in product_ids {
                if let Some(product) = self.get_product(product_id) {
                    let product = product.clone();
                    let store = &self.player.stores[store_idx];

                    if let Some(item) = store.inventory.get(&product_id) {
                        let retail_price = item.retail_price;
                        let available = item.quantity;

                        if available > 0 {
                            let sales = self.market.calculate_sales(
                                &product,
                                retail_price,
                                available,
                                customer_count,
                            );

                            if sales > 0 {
                                if let Some(revenue) =
                                    self.player.stores[store_idx].sell(product_id, sales)
                                {
                                    self.player.earn(revenue);
                                    total_revenue += revenue;
                                    total_items_sold += sales;
                                    sales_by_product.push((product.name.clone(), sales, revenue));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process each factory
        let factory_count = self.player.factories.len();
        let mut auto_transfers: Vec<(String, String, String, u32)> = Vec::new();

        for factory_idx in 0..factory_count {
            // Calculate expenses for this factory
            let factory = &self.player.factories[factory_idx];
            let rent = factory.daily_rent;
            let salaries: f64 = factory.workers.iter().map(|w| w.salary).sum();
            let factory_name = factory.name.clone();
            let factory_expenses = rent + salaries;
            total_expenses += factory_expenses;
            expenses_by_factory.push((factory_name.clone(), rent, salaries));

            // Advance production and collect completed items
            let completed = self.player.factories[factory_idx].advance_production();
            production_completed.extend(completed);

            // Process auto-transfers if enabled
            let factory = &self.player.factories[factory_idx];
            if factory.auto_transfer && !factory.connected_stores.is_empty() {
                // Get primary store for auto-transfer
                if let Some(primary_store_id) = factory.primary_store() {
                    // Find store index
                    if let Some(store_idx) = self.player.stores.iter().position(|s| s.id == primary_store_id) {
                        let store_name = self.player.stores[store_idx].name.clone();

                        // Transfer all finished goods
                        let product_ids: Vec<u32> = self.player.factories[factory_idx]
                            .finished_goods
                            .keys()
                            .copied()
                            .collect();

                        for product_id in product_ids {
                            let quantity = self.player.factories[factory_idx].get_finished_good(product_id);
                            if quantity > 0 {
                                // Get product info for retail price
                                if let Some(product) = self.get_product(product_id) {
                                    let product_name = product.name.clone();
                                    let retail_price = Market::suggest_retail_price(product.base_price, 50.0);

                                    // Take from factory and add to store
                                    if let Ok(transferred) = self.player.factories[factory_idx]
                                        .take_finished_goods(product_id, quantity)
                                    {
                                        self.player.stores[store_idx]
                                            .add_inventory(product_id, transferred, retail_price);
                                        auto_transfers.push((
                                            factory_name.clone(),
                                            store_name.clone(),
                                            product_name,
                                            transferred,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Deduct expenses
        self.player.cash -= total_expenses;

        // ==================== LOAN PROCESSING ====================

        // 1. Accrue interest on all loans
        for loan in &mut self.player.loans {
            let old_balance = loan.balance;
            loan.accrue_interest();
            loan_interest_accrued += loan.balance - old_balance;
        }

        // 2. Process auto-payments for line of credit loans
        let loan_ids: Vec<u32> = self.player.loans.iter().map(|l| l.id).collect();
        for loan_id in loan_ids {
            if let Some(loan) = self.player.get_loan(loan_id) {
                if loan.loan_type == LoanType::LineOfCredit {
                    let auto_payment = loan.get_auto_payment();
                    if auto_payment > 0.0 && self.player.cash >= auto_payment {
                        if let Some(paid) = self.player.make_loan_payment(loan_id, auto_payment) {
                            loan_payments.push((loan_id, paid));
                        }
                    } else if auto_payment > 0.0 {
                        // Can't afford auto-payment, pay what we can
                        let available = self.player.cash.max(0.0);
                        if available > 0.0 {
                            if let Some(paid) = self.player.make_loan_payment(loan_id, available) {
                                loan_payments.push((loan_id, paid));
                            }
                        }
                    }
                }
            }
        }

        // 3. Decrement days remaining on term loans and check for due loans
        for loan in &mut self.player.loans {
            if loan.loan_type == LoanType::TermLoan {
                loan.decrement_days();
            }
        }

        // 4. Check for due term loans
        let due_loan_ids: Vec<(u32, f64)> = self.player.loans
            .iter()
            .filter(|l| l.is_due())
            .map(|l| (l.id, l.balance))
            .collect();

        for (loan_id, balance) in due_loan_ids {
            loans_due.push((loan_id, balance));

            // Try to pay off the term loan
            if self.player.cash >= balance {
                self.player.make_loan_payment(loan_id, balance);
            } else {
                // Can't pay - apply penalty and pay what we can
                let penalty = self.player.get_loan(loan_id)
                    .map(|l| l.default_penalty())
                    .unwrap_or(0.0);
                term_loan_penalties += penalty;

                // Pay what we can
                let available = self.player.cash.max(0.0);
                if available > 0.0 {
                    self.player.make_loan_payment(loan_id, available);
                }

                // Add penalty to the loan balance
                if let Some(loan) = self.player.get_loan_mut(loan_id) {
                    loan.balance += penalty;
                }
            }
        }

        // 5. Collect warnings for loans coming due soon
        for loan in &self.player.loans {
            if let Some(days) = loan.is_due_soon() {
                loans_due_soon.push((loan.id, days, loan.balance));
            }
        }

        // 6. Clean up paid-off loans
        self.player.cleanup_loans();

        // Check for bankruptcy
        if self.player.cash < 0.0 {
            self.is_bankrupt = true;
        }

        self.day += 1;

        let net_profit = total_revenue - total_expenses - loan_interest_accrued;

        DayResult {
            total_revenue,
            total_items_sold,
            sales_by_product,
            total_expenses,
            expenses_by_store,
            expenses_by_factory,
            production_completed,
            net_profit,
            economic_state,
            economic_change,
            loan_interest_accrued,
            loan_payments,
            loans_due,
            loans_due_soon,
            term_loan_penalties,
            auto_transfers,
            competitor_events,
            player_market_share,
        }
    }

    /// Calculates average markup across all stores
    fn calculate_average_markup(&self) -> f64 {
        let mut total_markup = 0.0;
        let mut item_count = 0;

        for store in &self.player.stores {
            for (product_id, item) in &store.inventory {
                if let Some(product) = self.get_product(*product_id) {
                    let markup = ((item.retail_price - product.base_price) / product.base_price) * 100.0;
                    total_markup += markup;
                    item_count += 1;
                }
            }
        }

        if item_count > 0 {
            total_markup / item_count as f64
        } else {
            50.0 // Default markup if no inventory
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
