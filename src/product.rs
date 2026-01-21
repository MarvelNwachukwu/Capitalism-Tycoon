/// Represents the type of product
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductType {
    RawMaterial,      // Can only be used in manufacturing, not sold retail
    RetailGood,       // Regular products sold in stores
    ManufacturedGood, // Made in factories, sold in stores
}

impl ProductType {
    /// Returns true if this product can be sold in stores
    pub fn can_sell_retail(&self) -> bool {
        matches!(self, ProductType::RetailGood | ProductType::ManufacturedGood)
    }

    /// Returns true if this product can be used in manufacturing
    pub fn is_raw_material(&self) -> bool {
        matches!(self, ProductType::RawMaterial)
    }
}

/// Represents a category of products in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Food,
    Electronics,
    Clothing,
    RawMaterial,
    Furniture,
}

impl Category {
    /// Returns all retail categories (excluding raw materials)
    pub fn retail_categories() -> Vec<Category> {
        vec![
            Category::Food,
            Category::Electronics,
            Category::Clothing,
            Category::Furniture,
        ]
    }

    /// Returns all available categories
    pub fn all() -> Vec<Category> {
        vec![
            Category::Food,
            Category::Electronics,
            Category::Clothing,
            Category::RawMaterial,
            Category::Furniture,
        ]
    }

    /// Returns the display name for the category
    pub fn name(&self) -> &'static str {
        match self {
            Category::Food => "Food",
            Category::Electronics => "Electronics",
            Category::Clothing => "Clothing",
            Category::RawMaterial => "Raw Material",
            Category::Furniture => "Furniture",
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
    pub product_type: ProductType,
}

impl Product {
    /// Creates a new product (defaults to RetailGood)
    pub fn new(id: u32, name: &str, base_price: f64, category: Category) -> Self {
        Product {
            id,
            name: name.to_string(),
            base_price,
            category,
            product_type: ProductType::RetailGood,
        }
    }

    /// Creates a new product with a specific type
    pub fn new_with_type(
        id: u32,
        name: &str,
        base_price: f64,
        category: Category,
        product_type: ProductType,
    ) -> Self {
        Product {
            id,
            name: name.to_string(),
            base_price,
            category,
            product_type,
        }
    }

    /// Returns the default set of products available in the game
    pub fn default_products() -> Vec<Product> {
        vec![
            // Food items (IDs 1-4)
            Product::new(1, "Bread", 2.00, Category::Food),
            Product::new(2, "Milk", 3.50, Category::Food),
            Product::new(3, "Cheese", 5.00, Category::Food),
            Product::new(4, "Apples", 4.00, Category::Food),
            // Electronics (IDs 5-7)
            Product::new(5, "Headphones", 25.00, Category::Electronics),
            Product::new(6, "Phone Charger", 15.00, Category::Electronics),
            Product::new(7, "USB Cable", 8.00, Category::Electronics),
            // Clothing (IDs 8-10)
            Product::new(8, "T-Shirt", 12.00, Category::Clothing),
            Product::new(9, "Jeans", 35.00, Category::Clothing),
            Product::new(10, "Socks (3-pack)", 6.00, Category::Clothing),
            // Raw Materials (IDs 11-15)
            Product::new_with_type(11, "Lumber", 5.00, Category::RawMaterial, ProductType::RawMaterial),
            Product::new_with_type(12, "Steel", 8.00, Category::RawMaterial, ProductType::RawMaterial),
            Product::new_with_type(13, "Fabric", 4.00, Category::RawMaterial, ProductType::RawMaterial),
            Product::new_with_type(14, "Plastic", 3.00, Category::RawMaterial, ProductType::RawMaterial),
            Product::new_with_type(15, "Electronic Components", 15.00, Category::RawMaterial, ProductType::RawMaterial),
            // Manufactured Goods (IDs 16-21)
            Product::new_with_type(16, "Wooden Chair", 25.00, Category::Furniture, ProductType::ManufacturedGood),
            Product::new_with_type(17, "Steel Table", 60.00, Category::Furniture, ProductType::ManufacturedGood),
            Product::new_with_type(18, "Designer Jacket", 45.00, Category::Clothing, ProductType::ManufacturedGood),
            Product::new_with_type(19, "Blender", 55.00, Category::Electronics, ProductType::ManufacturedGood),
            Product::new_with_type(20, "Smartphone", 150.00, Category::Electronics, ProductType::ManufacturedGood),
            Product::new_with_type(21, "Laptop", 400.00, Category::Electronics, ProductType::ManufacturedGood),
        ]
    }

    /// Returns only retail products (excludes raw materials)
    pub fn retail_products() -> Vec<Product> {
        Self::default_products()
            .into_iter()
            .filter(|p| p.product_type.can_sell_retail())
            .collect()
    }

    /// Returns only raw materials
    pub fn raw_materials() -> Vec<Product> {
        Self::default_products()
            .into_iter()
            .filter(|p| p.product_type.is_raw_material())
            .collect()
    }

    /// Returns only manufactured goods
    pub fn manufactured_goods() -> Vec<Product> {
        Self::default_products()
            .into_iter()
            .filter(|p| matches!(p.product_type, ProductType::ManufacturedGood))
            .collect()
    }
}
