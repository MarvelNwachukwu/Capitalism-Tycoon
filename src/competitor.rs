/// Pricing strategy for AI competitors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PricingStrategy {
    /// Undercuts market prices to gain share
    Aggressive,
    /// Matches market prices
    Neutral,
    /// Premium pricing, fewer sales but higher margin
    Premium,
}

impl PricingStrategy {
    /// Returns the price multiplier for this strategy
    pub fn price_multiplier(&self) -> f64 {
        match self {
            PricingStrategy::Aggressive => 0.85,  // 15% below market
            PricingStrategy::Neutral => 1.0,      // Market price
            PricingStrategy::Premium => 1.20,     // 20% above market
        }
    }

    /// Returns customer attraction multiplier (inverse of price effect)
    pub fn attraction_multiplier(&self) -> f64 {
        match self {
            PricingStrategy::Aggressive => 1.3,   // More customers due to low prices
            PricingStrategy::Neutral => 1.0,
            PricingStrategy::Premium => 0.7,      // Fewer customers, but higher margin
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PricingStrategy::Aggressive => "Aggressive",
            PricingStrategy::Neutral => "Neutral",
            PricingStrategy::Premium => "Premium",
        }
    }
}

/// Represents an AI competitor business
#[derive(Debug, Clone)]
pub struct Competitor {
    pub id: u32,
    pub name: String,
    /// Number of stores (affects market share)
    pub store_count: u32,
    /// Average store quality (1.0 = standard, higher = better)
    pub store_quality: f64,
    /// Current pricing strategy
    pub strategy: PricingStrategy,
    /// Simulated cash for expansion decisions
    pub cash: f64,
    /// Base market share (0.0 to 1.0)
    base_share: f64,
    /// Days since last expansion
    days_since_expansion: u32,
}

impl Competitor {
    /// Creates a new competitor
    pub fn new(id: u32, name: &str, store_count: u32, strategy: PricingStrategy) -> Self {
        Competitor {
            id,
            name: name.to_string(),
            store_count,
            store_quality: 1.0,
            strategy,
            cash: 10000.0 + (store_count as f64 * 5000.0),
            base_share: 0.0,
            days_since_expansion: 0,
        }
    }

    /// Creates default competitors for a new game
    pub fn default_competitors() -> Vec<Competitor> {
        vec![
            Competitor::new(1, "MegaMart", 3, PricingStrategy::Aggressive),
            Competitor::new(2, "Quality Goods Co", 2, PricingStrategy::Premium),
            Competitor::new(3, "ValueStore", 2, PricingStrategy::Neutral),
        ]
    }

    /// Calculates this competitor's market power (used for share calculation)
    pub fn market_power(&self) -> f64 {
        let store_power = self.store_count as f64;
        let quality_bonus = self.store_quality;
        let strategy_bonus = self.strategy.attraction_multiplier();

        store_power * quality_bonus * strategy_bonus
    }

    /// Simulates one day of competitor activity
    /// Returns a message if something notable happened
    pub fn advance_day(&mut self, economic_multiplier: f64, player_market_share: f64) -> Option<String> {
        self.days_since_expansion += 1;

        // Earn simulated revenue based on market share and economy
        let daily_revenue = self.store_count as f64 * 200.0 * economic_multiplier * (1.0 - player_market_share);
        let daily_expenses = self.store_count as f64 * 150.0;
        self.cash += daily_revenue - daily_expenses;

        // Consider strategy change based on market conditions
        if player_market_share > 0.4 && self.strategy != PricingStrategy::Aggressive {
            // Player is dominating, become more aggressive
            if self.days_since_expansion > 10 {
                self.strategy = PricingStrategy::Aggressive;
                return Some(format!("{} has switched to aggressive pricing!", self.name));
            }
        }

        // Consider expansion
        if self.cash > 15000.0 && self.days_since_expansion > 14 {
            // Random chance to expand (simulated with cash threshold)
            if self.cash > 20000.0 {
                self.cash -= 10000.0;
                self.store_count += 1;
                self.days_since_expansion = 0;
                return Some(format!("{} has opened a new store! (Now has {} stores)", self.name, self.store_count));
            }
        }

        // Improve quality over time
        if self.days_since_expansion > 7 && self.store_quality < 1.5 {
            self.store_quality += 0.01;
        }

        None
    }

    /// React to player opening a new store
    pub fn react_to_player_expansion(&mut self) -> Option<String> {
        // Competitors may respond to player expansion
        if self.strategy == PricingStrategy::Neutral && self.cash > 5000.0 {
            self.strategy = PricingStrategy::Aggressive;
            return Some(format!("{} is responding with lower prices!", self.name));
        }
        None
    }
}

/// Manages the competitive market
#[derive(Debug)]
pub struct CompetitiveMarket {
    pub competitors: Vec<Competitor>,
    /// Total market size (base customers across all businesses)
    pub total_market_size: u32,
    /// Player's calculated market share (0.0 to 1.0)
    pub player_market_share: f64,
}

impl CompetitiveMarket {
    /// Creates a new competitive market
    pub fn new() -> Self {
        CompetitiveMarket {
            competitors: Competitor::default_competitors(),
            total_market_size: 500, // Base market of 500 potential customers
            player_market_share: 0.15, // Player starts with 15% share
        }
    }

    /// Calculates market shares based on all participants
    pub fn calculate_market_shares(&mut self, player_store_count: u32, player_avg_markup: f64) {
        // Player's market power
        let player_price_factor = if player_avg_markup > 60.0 {
            0.7 // High prices reduce attraction
        } else if player_avg_markup < 30.0 {
            1.3 // Low prices increase attraction
        } else {
            1.0
        };
        let player_power = player_store_count as f64 * player_price_factor;

        // Total competitor power
        let competitor_power: f64 = self.competitors.iter().map(|c| c.market_power()).sum();

        // Total market power
        let total_power = player_power + competitor_power;

        if total_power > 0.0 {
            self.player_market_share = (player_power / total_power).clamp(0.05, 0.95);
        } else {
            self.player_market_share = 0.5;
        }

        // Update competitor base shares
        for competitor in &mut self.competitors {
            competitor.base_share = competitor.market_power() / total_power;
        }
    }

    /// Returns the customer multiplier for player stores based on market share
    pub fn player_customer_multiplier(&self) -> f64 {
        // Market share affects how many of the potential customers come to player
        // Base multiplier scales with market share
        (self.player_market_share * 2.0).clamp(0.3, 1.5)
    }

    /// Advances all competitors by one day
    /// Returns notable events
    pub fn advance_day(&mut self, economic_multiplier: f64) -> Vec<String> {
        let player_share = self.player_market_share;
        let mut events = Vec::new();

        for competitor in &mut self.competitors {
            if let Some(event) = competitor.advance_day(economic_multiplier, player_share) {
                events.push(event);
            }
        }

        events
    }

    /// Gets total competitor store count
    pub fn total_competitor_stores(&self) -> u32 {
        self.competitors.iter().map(|c| c.store_count).sum()
    }

    /// Gets the leading competitor
    pub fn market_leader(&self) -> Option<&Competitor> {
        self.competitors.iter().max_by(|a, b| {
            a.market_power().partial_cmp(&b.market_power()).unwrap()
        })
    }

    /// Notify competitors of player expansion
    pub fn notify_player_expansion(&mut self) -> Vec<String> {
        let mut events = Vec::new();
        for competitor in &mut self.competitors {
            if let Some(event) = competitor.react_to_player_expansion() {
                events.push(event);
            }
        }
        events
    }
}

impl Default for CompetitiveMarket {
    fn default() -> Self {
        Self::new()
    }
}
