use std::collections::HashMap;

/// Represents an item in the store's inventory
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub product_id: u32,
    pub quantity: u32,
    pub retail_price: f64,
}

impl InventoryItem {
    /// Creates a new inventory item
    pub fn new(product_id: u32, quantity: u32, retail_price: f64) -> Self {
        InventoryItem {
            product_id,
            quantity,
            retail_price,
        }
    }
}

/// Represents a retail store
#[derive(Debug)]
pub struct Store {
    pub name: String,
    pub inventory: HashMap<u32, InventoryItem>,
    pub daily_customers: u32,
}

impl Store {
    /// Creates a new store with the given name
    pub fn new(name: &str) -> Self {
        Store {
            name: name.to_string(),
            inventory: HashMap::new(),
            daily_customers: 50, // Base number of daily customers
        }
    }

    /// Adds inventory to the store
    pub fn add_inventory(&mut self, product_id: u32, quantity: u32, retail_price: f64) {
        if let Some(item) = self.inventory.get_mut(&product_id) {
            item.quantity += quantity;
        } else {
            self.inventory.insert(
                product_id,
                InventoryItem::new(product_id, quantity, retail_price),
            );
        }
    }

    /// Sets the retail price for a product
    pub fn set_price(&mut self, product_id: u32, new_price: f64) -> bool {
        if let Some(item) = self.inventory.get_mut(&product_id) {
            item.retail_price = new_price;
            true
        } else {
            false
        }
    }

    /// Sells a quantity of a product, returns the revenue
    pub fn sell(&mut self, product_id: u32, quantity: u32) -> Option<f64> {
        if let Some(item) = self.inventory.get_mut(&product_id) {
            let actual_quantity = quantity.min(item.quantity);
            if actual_quantity > 0 {
                item.quantity -= actual_quantity;
                return Some(item.retail_price * actual_quantity as f64);
            }
        }
        None
    }

    /// Gets the quantity of a product in inventory
    pub fn get_quantity(&self, product_id: u32) -> u32 {
        self.inventory
            .get(&product_id)
            .map(|item| item.quantity)
            .unwrap_or(0)
    }

    /// Gets the retail price of a product
    pub fn get_price(&self, product_id: u32) -> Option<f64> {
        self.inventory.get(&product_id).map(|item| item.retail_price)
    }

    /// Returns total inventory value at retail prices
    pub fn total_inventory_value(&self) -> f64 {
        self.inventory
            .values()
            .map(|item| item.retail_price * item.quantity as f64)
            .sum()
    }

    /// Returns total number of items in inventory
    pub fn total_items(&self) -> u32 {
        self.inventory.values().map(|item| item.quantity).sum()
    }
}
