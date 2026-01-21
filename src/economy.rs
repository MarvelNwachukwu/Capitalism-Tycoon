use crate::product::{Category, Product};
use std::collections::HashMap;

/// Represents the market conditions
#[derive(Debug)]
pub struct Market {
    /// Current wholesale prices (product_id -> price)
    pub wholesale_prices: HashMap<u32, f64>,
    /// Base demand for each product category
    pub category_demand: HashMap<Category, f64>,
    /// Random seed for daily fluctuations
    day_seed: u64,
}

impl Market {
    /// Creates a new market with products
    pub fn new(products: &[Product]) -> Self {
        let mut wholesale_prices = HashMap::new();
        let mut category_demand = HashMap::new();

        for product in products {
            wholesale_prices.insert(product.id, product.base_price);
        }

        // Set base demand for each category
        category_demand.insert(Category::Food, 1.2); // Food sells well
        category_demand.insert(Category::Electronics, 0.8); // Electronics are specialty
        category_demand.insert(Category::Clothing, 1.0); // Clothing is average
        category_demand.insert(Category::Furniture, 0.6); // Furniture is less frequent purchase
        category_demand.insert(Category::RawMaterial, 0.0); // Raw materials can't be sold retail

        Market {
            wholesale_prices,
            category_demand,
            day_seed: 12345,
        }
    }

    /// Gets the wholesale price for a product
    pub fn get_wholesale_price(&self, product_id: u32) -> Option<f64> {
        self.wholesale_prices.get(&product_id).copied()
    }

    /// Updates market conditions for a new day
    pub fn advance_day(&mut self, day: u32) {
        self.day_seed = day as u64 * 31337 + 42;
    }

    /// Calculates expected sales based on price vs base price and demand
    /// Returns the number of units that would sell
    pub fn calculate_sales(
        &self,
        product: &Product,
        retail_price: f64,
        available_quantity: u32,
        customer_count: u32,
    ) -> u32 {
        let base_price = product.base_price;
        let category_multiplier = self
            .category_demand
            .get(&product.category)
            .copied()
            .unwrap_or(1.0);

        // Price elasticity: higher price = fewer sales
        // Formula: sales_factor = 1 - (price - base_price) / base_price * 0.5
        let price_ratio = (retail_price - base_price) / base_price;
        let price_factor = (1.0 - price_ratio * 0.5).clamp(0.0, 2.0);

        // Base demand per customer (small fraction of customers buy each product)
        let base_demand = 0.1 * category_multiplier;

        // Calculate expected sales
        let expected_sales = (customer_count as f64 * base_demand * price_factor) as u32;

        // Add some variance using simple pseudo-random
        let variance = self.get_daily_variance();
        let adjusted_sales = ((expected_sales as f64) * variance) as u32;

        // Can't sell more than we have
        adjusted_sales.min(available_quantity)
    }

    /// Returns a daily variance multiplier (0.8 to 1.2)
    fn get_daily_variance(&self) -> f64 {
        // Simple pseudo-random based on day seed
        let x = self.day_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let normalized = (x % 1000) as f64 / 1000.0; // 0.0 to 1.0
        0.8 + normalized * 0.4 // 0.8 to 1.2
    }

    /// Calculates the markup percentage
    pub fn calculate_markup(wholesale: f64, retail: f64) -> f64 {
        if wholesale > 0.0 {
            ((retail - wholesale) / wholesale) * 100.0
        } else {
            0.0
        }
    }

    /// Suggests a retail price based on markup percentage
    pub fn suggest_retail_price(wholesale: f64, markup_percent: f64) -> f64 {
        wholesale * (1.0 + markup_percent / 100.0)
    }
}
