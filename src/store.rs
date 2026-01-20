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

/// Represents an employee working at a store
#[derive(Debug, Clone)]
pub struct Employee {
    pub name: String,
    pub salary: f64,
}

impl Employee {
    /// Creates a new employee with the given name and default salary
    pub fn new(name: &str) -> Self {
        Employee {
            name: name.to_string(),
            salary: 50.0, // $50/day default salary
        }
    }
}

/// Represents a retail store
#[derive(Debug)]
pub struct Store {
    pub id: u32,
    pub name: String,
    pub inventory: HashMap<u32, InventoryItem>,
    pub daily_customers: u32,
    pub employees: Vec<Employee>,
    pub daily_rent: f64,
}

impl Store {
    /// Creates a new store with the given name and ID
    pub fn new(id: u32, name: &str) -> Self {
        Store {
            id,
            name: name.to_string(),
            inventory: HashMap::new(),
            daily_customers: 50, // Base number of daily customers
            employees: Vec::new(),
            daily_rent: 100.0, // $100/day default rent
        }
    }

    /// Hires a new employee (max 3 employees per store)
    pub fn hire_employee(&mut self, name: &str) -> Result<(), String> {
        if self.employees.len() >= 3 {
            return Err("Maximum of 3 employees per store".to_string());
        }
        self.employees.push(Employee::new(name));
        Ok(())
    }

    /// Fires an employee by index
    pub fn fire_employee(&mut self, index: usize) -> Result<Employee, String> {
        if index >= self.employees.len() {
            return Err("Invalid employee index".to_string());
        }
        Ok(self.employees.remove(index))
    }

    /// Calculates total daily expenses (rent + salaries)
    pub fn daily_expenses(&self) -> f64 {
        let total_salaries: f64 = self.employees.iter().map(|e| e.salary).sum();
        self.daily_rent + total_salaries
    }

    /// Calculates effective customer count (base + employee bonus)
    /// Each employee adds 20% more customers, max 3 employees (+60%)
    pub fn effective_customers(&self) -> u32 {
        let bonus_multiplier = 1.0 + (self.employees.len() as f64 * 0.2);
        (self.daily_customers as f64 * bonus_multiplier) as u32
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
