# Capitalism Tycoon

A text-based retail business simulation game written in Rust. Start with a small store, buy products wholesale, sell them at retail prices, and grow your business empire.

## Overview

Capitalism Tycoon is a terminal-based economic simulation where you play as a retail entrepreneur. Your goal is to maximize profits by strategically purchasing inventory at wholesale prices, setting competitive retail prices, managing staff, and expanding your business across multiple stores.

## Features

- **Retail Management**: Buy products wholesale and set your own retail prices
- **Price Elasticity**: Higher prices reduce sales; find the optimal markup
- **Multiple Stores**: Expand your empire by purchasing additional locations
- **Staff Management**: Hire employees to increase customer traffic (+20% per employee)
- **Dynamic Market**: Daily sales variance and category-based demand modifiers
- **Manufacturing System**: Factories, raw materials, and recipes for producing goods (in development)
- **Bankruptcy Risk**: Manage your cash flow or face game over

## Getting Started

### Prerequisites

- Rust (edition 2024)
- Cargo

### Installation

```bash
git clone https://github.com/MarvelNwachukwu/Capitalism-Tycoon.git
cd Capitalism-Tycoon
cargo build --release
```

### Running the Game

```bash
cargo run
```

## Gameplay

### Starting Conditions

- **Starting Capital**: $1,000
- **Initial Store**: 1 store with 50 base daily customers
- **Daily Rent**: $100 per store

### Game Loop

1. **View Store Inventory**: Check your current stock, prices, and markup percentages
2. **Buy Wholesale Inventory**: Purchase products from the wholesale market
3. **Set Retail Prices**: Adjust prices to balance profit margins and sales volume
4. **Advance Day**: Simulate a day of sales and expenses
5. **Manage Stores**: View all stores, switch between them, or buy new locations
6. **Manage Staff**: Hire or fire employees to adjust customer traffic

### Products

The game features 21 products across 5 categories:

| Category | Products | Demand Modifier |
|----------|----------|-----------------|
| Food | Bread, Milk, Cheese, Apples | 1.2x (high demand) |
| Electronics | Headphones, Phone Charger, USB Cable | 0.8x (specialty) |
| Clothing | T-Shirt, Jeans, Socks | 1.0x (average) |
| Raw Materials | Lumber, Steel, Fabric, Plastic, Electronic Components | N/A (manufacturing) |
| Furniture | Wooden Chair, Steel Table | Manufactured goods |

### Pricing Strategy

The game uses a price elasticity formula:

```
sales_factor = 1 - ((retail_price - base_price) / base_price) * 0.5
```

- **At base price**: 100% of potential sales
- **50% markup**: 75% of potential sales  
- **100% markup**: 50% of potential sales

### Employees

- **Cost**: $50/day salary per employee
- **Benefit**: +20% customer traffic per employee
- **Maximum**: 3 employees per store

### Store Expansion

- **Cost**: $5,000 per new store
- Each store operates independently with its own inventory, staff, and customer base

### Manufacturing (In Development)

The game includes a factory system for producing manufactured goods:

| Recipe | Ingredients | Output | Production Time |
|--------|-------------|--------|-----------------|
| Wooden Chair | 2 Lumber | 1 Chair | 1 day |
| Steel Table | 2 Steel + 1 Lumber | 1 Table | 2 days |
| Designer Jacket | 3 Fabric | 1 Jacket | 1 day |
| Blender | 1 Steel + 1 Electronics | 1 Blender | 2 days |
| Smartphone | 2 Electronics + 1 Plastic | 1 Smartphone | 3 days |
| Laptop | 3 Electronics + 1 Steel + 1 Plastic | 1 Laptop | 3 days |

## Project Structure

```
capitalism_tycoon/
├── Cargo.toml
├── src/
│   ├── main.rs       # Game entry point and main loop
│   ├── lib.rs        # Module exports
│   ├── game.rs       # Core game state and day simulation
│   ├── player.rs     # Player data (cash, stores, factories)
│   ├── store.rs      # Store management and inventory
│   ├── product.rs    # Product definitions and categories
│   ├── economy.rs    # Market conditions and sales calculations
│   ├── factory.rs    # Manufacturing facilities
│   ├── recipe.rs     # Production recipes
│   └── ui.rs         # Terminal UI and user interaction
```

## Architecture

### Core Components

- **GameState**: Central game state containing player, market, products, and day counter
- **Player**: Manages cash, stores, and factories
- **Store**: Handles inventory, employees, and daily operations
- **Market**: Calculates wholesale prices, demand, and sales
- **Product**: Defines items with types (retail, raw material, manufactured)

### Key Mechanics

1. **Daily Simulation**: Each day processes all stores, calculates sales based on inventory, prices, and customer count, then deducts expenses
2. **Sales Calculation**: Uses price elasticity, category demand, and daily variance (0.8x - 1.2x)
3. **Net Worth**: Calculated as cash + total inventory value at retail prices

## Tips for Success

1. Start with high-demand products (Food category)
2. Keep markup moderate (50-75%) for steady sales
3. Hire employees early to increase customer traffic
4. Monitor daily expenses vs revenue
5. Expand to new stores only when cash reserves are healthy
6. Balance inventory levels - don't overstock slow-moving items

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.
