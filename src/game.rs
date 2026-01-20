use crate::economy::Market;
use crate::player::Player;
use crate::product::Product;

/// Represents the complete game state
pub struct GameState {
    pub day: u32,
    pub player: Player,
    pub market: Market,
    pub products: Vec<Product>,
    pub current_store: usize,
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
    pub net_profit: f64,
}

impl GameState {
    /// Creates a new game with default settings
    pub fn new() -> Self {
        let products = Product::default_products();
        let market = Market::new(&products);
        let player = Player::new(1000.0, "My First Store");

        GameState {
            day: 1,
            player,
            market,
            products,
            current_store: 0,
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
    pub fn buy_new_store(&mut self, name: &str) -> Result<(), String> {
        const NEW_STORE_COST: f64 = 5000.0;

        if self.player.cash < NEW_STORE_COST {
            return Err(format!(
                "Not enough cash! Need ${:.2}, have ${:.2}",
                NEW_STORE_COST, self.player.cash
            ));
        }

        self.player.spend(NEW_STORE_COST);
        self.player.add_store(name);
        Ok(())
    }

    /// Calculates total daily expenses across all stores
    pub fn total_daily_expenses(&self) -> f64 {
        self.player.stores.iter().map(|s| s.daily_expenses()).sum()
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

    /// Advances to the next day and simulates sales for ALL stores
    pub fn advance_day(&mut self) -> DayResult {
        self.market.advance_day(self.day);

        let mut total_revenue = 0.0;
        let mut total_items_sold = 0;
        let mut sales_by_product = Vec::new();
        let mut total_expenses = 0.0;
        let mut expenses_by_store = Vec::new();

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

            // Get customer count with employee bonus
            let customer_count = self.player.stores[store_idx].effective_customers();

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

        // Deduct expenses
        self.player.cash -= total_expenses;

        // Check for bankruptcy
        if self.player.cash < 0.0 {
            self.is_bankrupt = true;
        }

        self.day += 1;

        let net_profit = total_revenue - total_expenses;

        DayResult {
            total_revenue,
            total_items_sold,
            sales_by_product,
            total_expenses,
            expenses_by_store,
            net_profit,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
