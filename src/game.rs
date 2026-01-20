use crate::economy::Market;
use crate::player::Player;
use crate::product::Product;

/// Represents the complete game state
pub struct GameState {
    pub day: u32,
    pub player: Player,
    pub market: Market,
    pub products: Vec<Product>,
}

/// Result of simulating a day's sales
#[derive(Debug)]
pub struct DayResult {
    pub total_revenue: f64,
    pub total_items_sold: u32,
    pub sales_by_product: Vec<(String, u32, f64)>, // (name, quantity, revenue)
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
        }
    }

    /// Gets a product by ID
    pub fn get_product(&self, product_id: u32) -> Option<&Product> {
        self.products.iter().find(|p| p.id == product_id)
    }

    /// Buys inventory from the wholesale market
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
        self.player
            .store_mut()
            .add_inventory(product_id, quantity, suggested_retail);

        Ok(total_cost)
    }

    /// Sets the retail price for a product in the store
    pub fn set_retail_price(&mut self, product_id: u32, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Price must be positive".to_string());
        }

        if self.player.store_mut().set_price(product_id, price) {
            Ok(())
        } else {
            Err("Product not in inventory".to_string())
        }
    }

    /// Advances to the next day and simulates sales
    pub fn advance_day(&mut self) -> DayResult {
        self.market.advance_day(self.day);

        let mut total_revenue = 0.0;
        let mut total_items_sold = 0;
        let mut sales_by_product = Vec::new();

        let store = self.player.store();
        let customer_count = store.daily_customers;

        // Clone inventory keys to avoid borrow issues
        let product_ids: Vec<u32> = store.inventory.keys().copied().collect();

        for product_id in product_ids {
            if let Some(product) = self.get_product(product_id) {
                let product = product.clone();
                let store = self.player.store();

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
                            if let Some(revenue) = self.player.store_mut().sell(product_id, sales) {
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

        self.day += 1;

        DayResult {
            total_revenue,
            total_items_sold,
            sales_by_product,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
