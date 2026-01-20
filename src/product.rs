/// Represents a category of products in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Food,
    Electronics,
    Clothing,
}

impl Category {
    /// Returns all available categories
    pub fn all() -> Vec<Category> {
        vec![Category::Food, Category::Electronics, Category::Clothing]
    }

    /// Returns the display name for the category
    pub fn name(&self) -> &'static str {
        match self {
            Category::Food => "Food",
            Category::Electronics => "Electronics",
            Category::Clothing => "Clothing",
        }
    }
}

/// Represents a product that can be bought and sold
#[derive(Debug, Clone)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub base_price: f64,
    pub category: Category,
}

impl Product {
    /// Creates a new product
    pub fn new(id: u32, name: &str, base_price: f64, category: Category) -> Self {
        Product {
            id,
            name: name.to_string(),
            base_price,
            category,
        }
    }

    /// Returns the default set of products available in the game
    pub fn default_products() -> Vec<Product> {
        vec![
            // Food items
            Product::new(1, "Bread", 2.00, Category::Food),
            Product::new(2, "Milk", 3.50, Category::Food),
            Product::new(3, "Cheese", 5.00, Category::Food),
            Product::new(4, "Apples", 4.00, Category::Food),
            // Electronics
            Product::new(5, "Headphones", 25.00, Category::Electronics),
            Product::new(6, "Phone Charger", 15.00, Category::Electronics),
            Product::new(7, "USB Cable", 8.00, Category::Electronics),
            // Clothing
            Product::new(8, "T-Shirt", 12.00, Category::Clothing),
            Product::new(9, "Jeans", 35.00, Category::Clothing),
            Product::new(10, "Socks (3-pack)", 6.00, Category::Clothing),
        ]
    }
}
