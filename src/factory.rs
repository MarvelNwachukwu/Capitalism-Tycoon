use std::collections::HashMap;
use crate::recipe::Recipe;

/// Represents a production job in progress
#[derive(Debug, Clone)]
pub struct ProductionJob {
    pub recipe_id: u32,
    pub recipe_name: String,
    pub days_remaining: u32,
    pub output_product_id: u32,
    pub output_quantity: u32,
}

impl ProductionJob {
    pub fn new(recipe: &Recipe) -> Self {
        ProductionJob {
            recipe_id: recipe.id,
            recipe_name: recipe.name.clone(),
            days_remaining: recipe.production_days,
            output_product_id: recipe.output_product_id,
            output_quantity: recipe.output_quantity,
        }
    }
}

/// Represents a worker at a factory
#[derive(Debug, Clone)]
pub struct FactoryWorker {
    pub name: String,
    pub salary: f64,
}

impl FactoryWorker {
    pub fn new(name: &str) -> Self {
        FactoryWorker {
            name: name.to_string(),
            salary: 75.0, // $75/day
        }
    }
}

/// Represents a completed production result
#[derive(Debug, Clone)]
pub struct ProductionResult {
    pub recipe_name: String,
    pub product_id: u32,
    pub quantity: u32,
}

/// Represents a manufacturing factory
#[derive(Debug)]
pub struct Factory {
    pub id: u32,
    pub name: String,
    pub raw_materials: HashMap<u32, u32>,  // product_id -> quantity
    pub finished_goods: HashMap<u32, u32>, // product_id -> quantity
    pub production_queue: Vec<ProductionJob>,
    pub workers: Vec<FactoryWorker>,
    pub daily_rent: f64,
}

impl Factory {
    /// Creates a new factory
    pub fn new(id: u32, name: &str) -> Self {
        Factory {
            id,
            name: name.to_string(),
            raw_materials: HashMap::new(),
            finished_goods: HashMap::new(),
            production_queue: Vec::new(),
            workers: Vec::new(),
            daily_rent: 150.0, // $150/day
        }
    }

    /// Returns the number of available production slots (base 2 + 1 per worker)
    pub fn production_slots(&self) -> usize {
        2 + self.workers.len()
    }

    /// Returns the number of currently active production jobs
    pub fn active_jobs(&self) -> usize {
        self.production_queue.len()
    }

    /// Returns the number of available slots for new jobs
    pub fn available_slots(&self) -> usize {
        self.production_slots().saturating_sub(self.active_jobs())
    }

    /// Adds raw materials to the factory storage
    pub fn add_raw_material(&mut self, product_id: u32, quantity: u32) {
        *self.raw_materials.entry(product_id).or_insert(0) += quantity;
    }

    /// Gets the quantity of a raw material in storage
    pub fn get_raw_material(&self, product_id: u32) -> u32 {
        *self.raw_materials.get(&product_id).unwrap_or(&0)
    }

    /// Gets the quantity of a finished good in storage
    pub fn get_finished_good(&self, product_id: u32) -> u32 {
        *self.finished_goods.get(&product_id).unwrap_or(&0)
    }

    /// Checks if the factory has enough raw materials to produce a recipe
    pub fn has_ingredients(&self, recipe: &Recipe) -> bool {
        recipe.ingredients.iter().all(|ing| {
            self.get_raw_material(ing.product_id) >= ing.quantity
        })
    }

    /// Returns missing ingredients for a recipe (if any)
    pub fn missing_ingredients(&self, recipe: &Recipe) -> Vec<(u32, u32)> {
        recipe
            .ingredients
            .iter()
            .filter_map(|ing| {
                let have = self.get_raw_material(ing.product_id);
                if have < ing.quantity {
                    Some((ing.product_id, ing.quantity - have))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Starts production of a recipe, consuming raw materials
    pub fn start_production(&mut self, recipe: &Recipe) -> Result<(), String> {
        // Check available slots
        if self.available_slots() == 0 {
            return Err("No available production slots".to_string());
        }

        // Check ingredients
        if !self.has_ingredients(recipe) {
            return Err("Insufficient raw materials".to_string());
        }

        // Consume raw materials
        for ing in &recipe.ingredients {
            if let Some(qty) = self.raw_materials.get_mut(&ing.product_id) {
                *qty -= ing.quantity;
            }
        }

        // Add job to queue
        self.production_queue.push(ProductionJob::new(recipe));

        Ok(())
    }

    /// Advances all production jobs by one day, returns completed products
    pub fn advance_production(&mut self) -> Vec<ProductionResult> {
        let mut completed = Vec::new();
        let mut still_in_progress = Vec::new();

        for mut job in self.production_queue.drain(..) {
            job.days_remaining -= 1;
            if job.days_remaining == 0 {
                // Job complete - add to finished goods
                *self.finished_goods.entry(job.output_product_id).or_insert(0) +=
                    job.output_quantity;
                completed.push(ProductionResult {
                    recipe_name: job.recipe_name,
                    product_id: job.output_product_id,
                    quantity: job.output_quantity,
                });
            } else {
                still_in_progress.push(job);
            }
        }

        self.production_queue = still_in_progress;
        completed
    }

    /// Removes finished goods from factory storage (for transfer to store)
    pub fn take_finished_goods(&mut self, product_id: u32, quantity: u32) -> Result<u32, String> {
        let available = self.get_finished_good(product_id);
        if available == 0 {
            return Err("No finished goods of this type".to_string());
        }

        let actual_quantity = quantity.min(available);
        if let Some(qty) = self.finished_goods.get_mut(&product_id) {
            *qty -= actual_quantity;
        }

        Ok(actual_quantity)
    }

    /// Calculates total daily expenses (rent + worker salaries)
    pub fn daily_expenses(&self) -> f64 {
        let salaries: f64 = self.workers.iter().map(|w| w.salary).sum();
        self.daily_rent + salaries
    }

    /// Hires a new worker (max 3 workers per factory)
    pub fn hire_worker(&mut self, name: &str) -> Result<(), String> {
        if self.workers.len() >= 3 {
            return Err("Maximum of 3 workers per factory".to_string());
        }
        self.workers.push(FactoryWorker::new(name));
        Ok(())
    }

    /// Fires a worker by index
    pub fn fire_worker(&mut self, index: usize) -> Result<FactoryWorker, String> {
        if index >= self.workers.len() {
            return Err("Invalid worker index".to_string());
        }
        Ok(self.workers.remove(index))
    }

    /// Returns total raw material count
    pub fn total_raw_materials(&self) -> u32 {
        self.raw_materials.values().sum()
    }

    /// Returns total finished goods count
    pub fn total_finished_goods(&self) -> u32 {
        self.finished_goods.values().sum()
    }

    /// Calculates how many times a recipe can be produced given available slots and materials
    pub fn max_producible(&self, recipe: &Recipe) -> u32 {
        // Limited by available slots
        let slot_limit = self.available_slots() as u32;
        if slot_limit == 0 {
            return 0;
        }

        // Limited by ingredients - find the minimum batches we can make
        let material_limit = recipe
            .ingredients
            .iter()
            .map(|ing| {
                let have = self.get_raw_material(ing.product_id);
                have / ing.quantity
            })
            .min()
            .unwrap_or(0);

        slot_limit.min(material_limit)
    }

    /// Starts production of a recipe multiple times, consuming raw materials
    /// Returns the number of jobs actually started
    pub fn start_production_batch(&mut self, recipe: &Recipe, quantity: u32) -> Result<u32, String> {
        if quantity == 0 {
            return Err("Quantity must be greater than 0".to_string());
        }

        let max_possible = self.max_producible(recipe);
        if max_possible == 0 {
            if self.available_slots() == 0 {
                return Err("No available production slots".to_string());
            } else {
                return Err("Insufficient raw materials".to_string());
            }
        }

        let actual_quantity = quantity.min(max_possible);

        // Start each job
        for _ in 0..actual_quantity {
            // Consume raw materials
            for ing in &recipe.ingredients {
                if let Some(qty) = self.raw_materials.get_mut(&ing.product_id) {
                    *qty -= ing.quantity;
                }
            }
            // Add job to queue
            self.production_queue.push(ProductionJob::new(recipe));
        }

        Ok(actual_quantity)
    }
}
