use crate::store::Store;

/// Represents the player in the game
#[derive(Debug)]
pub struct Player {
    pub cash: f64,
    pub stores: Vec<Store>,
    next_store_id: u32,
}

impl Player {
    /// Creates a new player with starting cash and one store
    pub fn new(starting_cash: f64, store_name: &str) -> Self {
        Player {
            cash: starting_cash,
            stores: vec![Store::new(1, store_name)],
            next_store_id: 2,
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

    /// Returns the player's total net worth (cash + inventory value)
    pub fn net_worth(&self) -> f64 {
        let inventory_value: f64 = self.stores.iter().map(|s| s.total_inventory_value()).sum();
        self.cash + inventory_value
    }

    /// Returns the total daily expenses across all stores
    pub fn total_daily_expenses(&self) -> f64 {
        self.stores.iter().map(|s| s.daily_expenses()).sum()
    }
}
