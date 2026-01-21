use crate::product::{Category, Product};
use std::collections::HashMap;

/// Represents the current state of the economy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EconomicState {
    Collapse,
    Recession,
    Standard,
    Growth,
    Booming,
    Prosperity,
}

impl EconomicState {
    /// Returns the base interest rate for this economic state
    pub fn interest_rate(&self) -> f64 {
        match self {
            EconomicState::Collapse => 0.15,    // 15%
            EconomicState::Recession => 0.10,   // 10%
            EconomicState::Standard => 0.06,    // 6%
            EconomicState::Growth => 0.05,      // 5%
            EconomicState::Booming => 0.04,     // 4%
            EconomicState::Prosperity => 0.03,  // 3%
        }
    }

    /// Returns the sales multiplier (affects customer purchases)
    pub fn sales_multiplier(&self) -> f64 {
        match self {
            EconomicState::Collapse => 0.5,
            EconomicState::Recession => 0.7,
            EconomicState::Standard => 1.0,
            EconomicState::Growth => 1.2,
            EconomicState::Booming => 1.4,
            EconomicState::Prosperity => 1.6,
        }
    }

    /// Returns the wholesale price multiplier
    pub fn price_multiplier(&self) -> f64 {
        match self {
            EconomicState::Collapse => 0.8,
            EconomicState::Recession => 0.9,
            EconomicState::Standard => 1.0,
            EconomicState::Growth => 1.05,
            EconomicState::Booming => 1.1,
            EconomicState::Prosperity => 1.15,
        }
    }

    /// Returns the next state if transitioning up (toward prosperity)
    pub fn transition_up(&self) -> Option<EconomicState> {
        match self {
            EconomicState::Collapse => Some(EconomicState::Recession),
            EconomicState::Recession => Some(EconomicState::Standard),
            EconomicState::Standard => Some(EconomicState::Growth),
            EconomicState::Growth => Some(EconomicState::Booming),
            EconomicState::Booming => Some(EconomicState::Prosperity),
            EconomicState::Prosperity => None,
        }
    }

    /// Returns the next state if transitioning down (toward collapse)
    pub fn transition_down(&self) -> Option<EconomicState> {
        match self {
            EconomicState::Collapse => None,
            EconomicState::Recession => Some(EconomicState::Collapse),
            EconomicState::Standard => Some(EconomicState::Recession),
            EconomicState::Growth => Some(EconomicState::Standard),
            EconomicState::Booming => Some(EconomicState::Growth),
            EconomicState::Prosperity => Some(EconomicState::Booming),
        }
    }

    /// Returns a display name for the economic state
    pub fn name(&self) -> &'static str {
        match self {
            EconomicState::Collapse => "Collapse",
            EconomicState::Recession => "Recession",
            EconomicState::Standard => "Standard",
            EconomicState::Growth => "Growth",
            EconomicState::Booming => "Booming",
            EconomicState::Prosperity => "Prosperity",
        }
    }

    /// Returns a description of the economic state
    pub fn description(&self) -> &'static str {
        match self {
            EconomicState::Collapse => "Economic crisis, very hard times",
            EconomicState::Recession => "Economic downturn, reduced spending",
            EconomicState::Standard => "Normal economic conditions",
            EconomicState::Growth => "Expanding economy",
            EconomicState::Booming => "Strong economic growth",
            EconomicState::Prosperity => "Peak economic conditions",
        }
    }

    /// Returns true if this is an extreme state (for mean reversion)
    pub fn is_extreme(&self) -> bool {
        matches!(self, EconomicState::Collapse | EconomicState::Prosperity)
    }
}

impl std::fmt::Display for EconomicState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents the market conditions
#[derive(Debug)]
pub struct Market {
    /// Current wholesale prices (product_id -> price)
    pub wholesale_prices: HashMap<u32, f64>,
    /// Base demand for each product category
    pub category_demand: HashMap<Category, f64>,
    /// Random seed for daily fluctuations
    day_seed: u64,
    /// Current economic state
    pub economic_state: EconomicState,
    /// Economic trend (-1.0 to 1.0, affects transition probability)
    pub economic_trend: f64,
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
            economic_state: EconomicState::Standard,
            economic_trend: 0.0,
        }
    }

    /// Gets the wholesale price for a product, adjusted by economic state
    pub fn get_wholesale_price(&self, product_id: u32) -> Option<f64> {
        self.wholesale_prices
            .get(&product_id)
            .map(|&base_price| base_price * self.economic_state.price_multiplier())
    }

    /// Gets the base wholesale price without economic adjustment
    pub fn get_base_wholesale_price(&self, product_id: u32) -> Option<f64> {
        self.wholesale_prices.get(&product_id).copied()
    }

    /// Updates market conditions for a new day and returns any economic change
    pub fn advance_day(&mut self, day: u32) -> Option<String> {
        self.day_seed = day as u64 * 31337 + 42;
        self.update_economy(day)
    }

    /// Updates the economic state based on trend and random chance
    /// Returns a message if the state changed
    fn update_economy(&mut self, day: u32) -> Option<String> {
        let old_state = self.economic_state;

        // Update trend (slow sine wave over ~50 days)
        self.economic_trend = (day as f64 * 0.125).sin();

        // Base transition chances
        let mut up_chance = 0.04;   // 4% base chance to improve
        let mut down_chance = 0.04; // 4% base chance to worsen

        // Modify by trend
        if self.economic_trend > 0.0 {
            up_chance += self.economic_trend * 0.06;  // Up to +6%
        } else {
            down_chance += (-self.economic_trend) * 0.06;
        }

        // Mean reversion for extreme states
        match self.economic_state {
            EconomicState::Collapse => {
                up_chance += 0.10;
                down_chance = 0.0;
            }
            EconomicState::Prosperity => {
                down_chance += 0.10;
                up_chance = 0.0;
            }
            _ => {}
        }

        // Roll for transition using day-based pseudo-random
        let roll = self.get_random_value();
        if roll < up_chance {
            if let Some(new_state) = self.economic_state.transition_up() {
                self.economic_state = new_state;
            }
        } else if roll < up_chance + down_chance {
            if let Some(new_state) = self.economic_state.transition_down() {
                self.economic_state = new_state;
            }
        }

        // Return message if state changed
        if self.economic_state != old_state {
            let direction = if self.economic_state.sales_multiplier() > old_state.sales_multiplier() {
                "improved"
            } else {
                "worsened"
            };
            Some(format!(
                "Economy {} to {}!",
                direction,
                self.economic_state.name()
            ))
        } else {
            None
        }
    }

    /// Returns a random value between 0.0 and 1.0 based on current day seed
    fn get_random_value(&self) -> f64 {
        let x = self.day_seed.wrapping_mul(48271).wrapping_add(1);
        (x % 10000) as f64 / 10000.0
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

        // Apply economic state sales multiplier
        let economic_multiplier = self.economic_state.sales_multiplier();

        // Base demand per customer (small fraction of customers buy each product)
        let base_demand = 0.1 * category_multiplier * economic_multiplier;

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

    /// Gets the interest rate for a specific loan type based on current economy
    pub fn get_loan_rate(&self, loan_type: &crate::loan::LoanType) -> f64 {
        self.economic_state.interest_rate() + loan_type.rate_modifier()
    }
}
