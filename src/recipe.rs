/// Represents an ingredient required for a recipe
#[derive(Debug, Clone)]
pub struct RecipeIngredient {
    pub product_id: u32,
    pub quantity: u32,
}

impl RecipeIngredient {
    pub fn new(product_id: u32, quantity: u32) -> Self {
        RecipeIngredient { product_id, quantity }
    }
}

/// Represents a manufacturing recipe
#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: u32,
    pub name: String,
    pub ingredients: Vec<RecipeIngredient>,
    pub output_product_id: u32,
    pub output_quantity: u32,
    pub production_days: u32,
}

impl Recipe {
    /// Creates a new recipe
    pub fn new(
        id: u32,
        name: &str,
        ingredients: Vec<RecipeIngredient>,
        output_product_id: u32,
        output_quantity: u32,
        production_days: u32,
    ) -> Self {
        Recipe {
            id,
            name: name.to_string(),
            ingredients,
            output_product_id,
            output_quantity,
            production_days,
        }
    }

    /// Returns the default set of manufacturing recipes
    /// Product IDs:
    ///   Raw Materials: 11=Lumber, 12=Steel, 13=Fabric, 14=Plastic, 15=Electronics
    ///   Manufactured: 16=Chair, 17=Table, 18=Jacket, 19=Blender, 20=Smartphone, 21=Laptop
    pub fn default_recipes() -> Vec<Recipe> {
        vec![
            // Wooden Chair: 2 Lumber -> 1 Chair (1 day)
            Recipe::new(
                1,
                "Wooden Chair",
                vec![RecipeIngredient::new(11, 2)],
                16,
                1,
                1,
            ),
            // Steel Table: 2 Steel + 1 Lumber -> 1 Table (2 days)
            Recipe::new(
                2,
                "Steel Table",
                vec![
                    RecipeIngredient::new(12, 2),
                    RecipeIngredient::new(11, 1),
                ],
                17,
                1,
                2,
            ),
            // Designer Jacket: 3 Fabric -> 1 Jacket (1 day)
            Recipe::new(
                3,
                "Designer Jacket",
                vec![RecipeIngredient::new(13, 3)],
                18,
                1,
                1,
            ),
            // Blender: 1 Steel + 1 Electronics -> 1 Blender (2 days)
            Recipe::new(
                4,
                "Blender",
                vec![
                    RecipeIngredient::new(12, 1),
                    RecipeIngredient::new(15, 1),
                ],
                19,
                1,
                2,
            ),
            // Smartphone: 2 Electronics + 1 Plastic -> 1 Smartphone (3 days)
            Recipe::new(
                5,
                "Smartphone",
                vec![
                    RecipeIngredient::new(15, 2),
                    RecipeIngredient::new(14, 1),
                ],
                20,
                1,
                3,
            ),
            // Laptop: 3 Electronics + 1 Steel + 1 Plastic -> 1 Laptop (3 days)
            Recipe::new(
                6,
                "Laptop",
                vec![
                    RecipeIngredient::new(15, 3),
                    RecipeIngredient::new(12, 1),
                    RecipeIngredient::new(14, 1),
                ],
                21,
                1,
                3,
            ),
        ]
    }

    /// Calculates the total raw material cost for this recipe
    pub fn material_cost(&self, get_price: impl Fn(u32) -> f64) -> f64 {
        self.ingredients
            .iter()
            .map(|ing| get_price(ing.product_id) * ing.quantity as f64)
            .sum()
    }
}
