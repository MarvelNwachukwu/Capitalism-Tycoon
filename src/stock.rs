use crate::economy::EconomicState;

/// Type of stock determining risk/reward profile
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StockType {
    /// Stable, low risk, pays dividends
    BlueChip,
    /// Medium risk, moderate growth potential
    Growth,
    /// High risk, high potential gains/losses
    Speculative,
}

impl StockType {
    /// Returns the base volatility for this stock type (daily % swing)
    pub fn base_volatility(&self) -> f64 {
        match self {
            StockType::BlueChip => 0.02,     // 2% daily swing
            StockType::Growth => 0.05,       // 5% daily swing
            StockType::Speculative => 0.12,  // 12% daily swing
        }
    }

    /// Returns dividend yield (annual %, paid daily)
    pub fn dividend_yield(&self) -> f64 {
        match self {
            StockType::BlueChip => 0.04,     // 4% annual
            StockType::Growth => 0.01,       // 1% annual
            StockType::Speculative => 0.0,   // No dividends
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StockType::BlueChip => "Blue Chip",
            StockType::Growth => "Growth",
            StockType::Speculative => "Speculative",
        }
    }
}

/// Represents a tradeable stock
#[derive(Debug, Clone)]
pub struct Stock {
    pub id: u32,
    pub symbol: String,
    pub name: String,
    pub stock_type: StockType,
    pub price: f64,
    pub base_price: f64,
    /// Price history for trend calculation (last 7 days)
    price_history: Vec<f64>,
    /// Accumulated fractional price changes
    price_accumulator: f64,
}

impl Stock {
    pub fn new(id: u32, symbol: &str, name: &str, stock_type: StockType, price: f64) -> Self {
        Stock {
            id,
            symbol: symbol.to_string(),
            name: name.to_string(),
            stock_type,
            price,
            base_price: price,
            price_history: vec![price],
            price_accumulator: 0.0,
        }
    }

    /// Creates default stocks for a new game
    pub fn default_stocks() -> Vec<Stock> {
        vec![
            // Blue Chip stocks - stable, dividends
            Stock::new(1, "MEGA", "MegaCorp Industries", StockType::BlueChip, 100.0),
            Stock::new(2, "SAFE", "SafeHaven Holdings", StockType::BlueChip, 75.0),
            // Growth stocks - moderate risk
            Stock::new(3, "TECH", "TechGrowth Inc", StockType::Growth, 50.0),
            Stock::new(4, "RETL", "RetailExpand Co", StockType::Growth, 35.0),
            // Speculative stocks - high risk
            Stock::new(5, "MOON", "MoonShot Ventures", StockType::Speculative, 15.0),
            Stock::new(6, "RISK", "RiskyBet Gaming", StockType::Speculative, 8.0),
        ]
    }

    /// Updates stock price based on economy and randomness
    /// Returns the price change amount
    pub fn update_price(&mut self, economic_state: &EconomicState, random_factor: f64) -> f64 {
        let old_price = self.price;

        // Economic influence on stock prices
        let economic_trend = match economic_state {
            EconomicState::Collapse => -0.03,    // Strong downward pressure
            EconomicState::Recession => -0.015,  // Moderate downward
            EconomicState::Standard => 0.0,      // Neutral
            EconomicState::Growth => 0.01,       // Slight upward
            EconomicState::Booming => 0.02,      // Moderate upward
            EconomicState::Prosperity => 0.025,  // Strong upward
        };

        // Random component (-1.0 to 1.0 expected)
        let volatility = self.stock_type.base_volatility();
        let random_change = random_factor * volatility;

        // Combined change
        let total_change = economic_trend + random_change;

        // Apply change with mean reversion toward base price
        let reversion_strength = 0.01;
        let reversion = (self.base_price - self.price) / self.base_price * reversion_strength;

        // Accumulate the fractional change
        self.price_accumulator += self.price * (total_change + reversion);

        // Only apply changes when they accumulate to at least $0.01
        if self.price_accumulator.abs() >= 0.01 {
            let change = (self.price_accumulator * 100.0).round() / 100.0;
            self.price += change;
            self.price_accumulator -= change;
        }

        // Minimum price floor
        if self.price < 0.50 {
            self.price = 0.50;
        }

        // Update price history (keep last 7 days)
        self.price_history.push(self.price);
        if self.price_history.len() > 7 {
            self.price_history.remove(0);
        }

        self.price - old_price
    }

    /// Returns the 7-day price trend as a percentage
    pub fn trend(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }
        let oldest = self.price_history.first().unwrap();
        let newest = self.price_history.last().unwrap();
        ((newest - oldest) / oldest) * 100.0
    }

    /// Returns daily dividend per share
    pub fn daily_dividend(&self) -> f64 {
        self.price * self.stock_type.dividend_yield() / 365.0
    }

    /// Returns trend indicator string
    pub fn trend_indicator(&self) -> &'static str {
        let trend = self.trend();
        if trend > 5.0 {
            "▲▲"
        } else if trend > 1.0 {
            "▲"
        } else if trend < -5.0 {
            "▼▼"
        } else if trend < -1.0 {
            "▼"
        } else {
            "─"
        }
    }
}

/// Represents a player's holding in a stock
#[derive(Debug, Clone)]
pub struct StockHolding {
    pub stock_id: u32,
    pub shares: u32,
    pub avg_purchase_price: f64,
    pub total_dividends_earned: f64,
}

impl StockHolding {
    pub fn new(stock_id: u32, shares: u32, price: f64) -> Self {
        StockHolding {
            stock_id,
            shares,
            avg_purchase_price: price,
            total_dividends_earned: 0.0,
        }
    }

    /// Adds more shares, updating average price
    pub fn add_shares(&mut self, shares: u32, price: f64) {
        let total_cost = self.avg_purchase_price * self.shares as f64 + price * shares as f64;
        self.shares += shares;
        self.avg_purchase_price = total_cost / self.shares as f64;
    }

    /// Removes shares, returns true if successful
    pub fn remove_shares(&mut self, shares: u32) -> bool {
        if shares > self.shares {
            return false;
        }
        self.shares -= shares;
        true
    }

    /// Calculates current value at given market price
    pub fn current_value(&self, market_price: f64) -> f64 {
        market_price * self.shares as f64
    }

    /// Calculates total gain/loss at given market price
    pub fn gain_loss(&self, market_price: f64) -> f64 {
        self.current_value(market_price) - (self.avg_purchase_price * self.shares as f64)
    }

    /// Calculates gain/loss percentage
    pub fn gain_loss_percent(&self, market_price: f64) -> f64 {
        if self.shares == 0 {
            return 0.0;
        }
        ((market_price - self.avg_purchase_price) / self.avg_purchase_price) * 100.0
    }

    /// Records dividend payment
    pub fn receive_dividend(&mut self, amount: f64) {
        self.total_dividends_earned += amount;
    }
}

/// Manages the stock market
#[derive(Debug)]
pub struct StockMarket {
    pub stocks: Vec<Stock>,
    /// Simple pseudo-random state for price fluctuations
    random_state: u64,
}

impl StockMarket {
    pub fn new() -> Self {
        StockMarket {
            stocks: Stock::default_stocks(),
            random_state: 12345,
        }
    }

    /// Gets a stock by ID
    pub fn get_stock(&self, stock_id: u32) -> Option<&Stock> {
        self.stocks.iter().find(|s| s.id == stock_id)
    }

    /// Gets a mutable stock by ID
    pub fn get_stock_mut(&mut self, stock_id: u32) -> Option<&mut Stock> {
        self.stocks.iter_mut().find(|s| s.id == stock_id)
    }

    /// Simple pseudo-random number generator (-1.0 to 1.0)
    fn next_random(&mut self) -> f64 {
        // Linear congruential generator
        self.random_state = self.random_state.wrapping_mul(1103515245).wrapping_add(12345);
        let value = ((self.random_state >> 16) & 0x7FFF) as f64 / 32767.0;
        value * 2.0 - 1.0 // Convert to -1.0 to 1.0
    }

    /// Updates all stock prices for a new day
    /// Returns list of (stock_symbol, old_price, new_price, change)
    pub fn advance_day(&mut self, economic_state: &EconomicState) -> Vec<(String, f64, f64, f64)> {
        // Generate random numbers first to avoid borrow issues
        let randoms: Vec<f64> = (0..self.stocks.len())
            .map(|_| self.next_random())
            .collect();

        let mut changes = Vec::new();

        for (i, stock) in self.stocks.iter_mut().enumerate() {
            let old_price = stock.price;
            let random = randoms[i];
            let change = stock.update_price(economic_state, random);
            changes.push((stock.symbol.clone(), old_price, stock.price, change));
        }

        changes
    }

    /// Gets total market value of all stocks (market cap simulation)
    pub fn total_market_value(&self) -> f64 {
        self.stocks.iter().map(|s| s.price * 1000.0).sum() // Assume 1000 shares per stock
    }
}

impl Default for StockMarket {
    fn default() -> Self {
        Self::new()
    }
}
