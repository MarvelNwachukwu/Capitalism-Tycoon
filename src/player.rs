use crate::store::Store;

/// Represents the player in the game
#[derive(Debug)]
pub struct Player {
    pub cash: f64,
    pub stores: Vec<Store>,
}

impl Player {
    /// Creates a new player with starting cash and one store
    pub fn new(starting_cash: f64, store_name: &str) -> Self {
        Player {
            cash: starting_cash,
            stores: vec![Store::new(store_name)],
        }
    }

    /// Gets a reference to the player's first (and for now, only) store
    pub fn store(&self) -> &Store {
        &self.stores[0]
    }

    /// Gets a mutable reference to the player's first store
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.stores[0]
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
}
