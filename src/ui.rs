use crate::economy::Market;
use crate::game::{DayResult, GameState};
use crate::loan::{Loan, LoanType};
use crate::product::Product;
use std::io::{self, Write};

/// Menu options for the main game loop
#[derive(Debug, PartialEq)]
pub enum MenuChoice {
    ViewStore,
    BuyInventory,
    SetPrices,
    AdvanceDay,
    ManageStores,
    ManageStaff,
    ManageFactories,
    ManageLoans,
    ManageInvestments,
    Quit,
}

/// Clears the screen (simple version)
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Displays the game header with status
pub fn display_header(game: &GameState) {
    let current_store = game.current_store();
    let daily_expenses = game.total_daily_expenses();
    let economic_state = &game.market.economic_state;
    let total_debt = game.player.total_debt();
    let market_share = game.competitive_market.player_market_share * 100.0;
    let portfolio_value = game.portfolio_value();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              BUSINESS TYCOON - Rust Edition                  ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Day: {:>4}  │  Cash: ${:>10.2}  │  Net Worth: ${:>10.2}  ║",
        game.day,
        game.player.cash,
        game.player.net_worth()
    );
    println!(
        "║  Store: {:16} │  Daily Expenses: ${:>10.2}   ║",
        current_store.name,
        daily_expenses
    );
    println!(
        "║  Economy: {:12} │  Market Share: {:>5.1}%             ║",
        economic_state.name(),
        market_share
    );
    println!(
        "║  Debt: ${:>10.2}    │  Portfolio: ${:>10.2}          ║",
        total_debt,
        portfolio_value
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

/// Displays the main menu and returns the user's choice
pub fn display_menu() -> MenuChoice {
    println!("What would you like to do?");
    println!("  [1] View store inventory");
    println!("  [2] Buy wholesale inventory");
    println!("  [3] Set retail prices");
    println!("  [4] Advance to next day (simulate sales)");
    println!("  [5] Manage stores");
    println!("  [6] Manage staff");
    println!("  [7] Manage factories");
    println!("  [8] Manage loans");
    println!("  [9] Manage investments");
    println!("  [0] Quit game");
    println!();

    loop {
        let input = read_input("Enter choice (0-9): ");
        match input.trim() {
            "1" => return MenuChoice::ViewStore,
            "2" => return MenuChoice::BuyInventory,
            "3" => return MenuChoice::SetPrices,
            "4" => return MenuChoice::AdvanceDay,
            "5" => return MenuChoice::ManageStores,
            "6" => return MenuChoice::ManageStaff,
            "7" => return MenuChoice::ManageFactories,
            "8" => return MenuChoice::ManageLoans,
            "9" => return MenuChoice::ManageInvestments,
            "0" => return MenuChoice::Quit,
            _ => println!("Invalid choice. Please enter 0-9."),
        }
    }
}

/// Displays the store inventory
pub fn display_store(game: &GameState) {
    let store = game.current_store();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║  {:^58}  ║",
        format!("{} - Inventory", store.name)
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Employees: {}  │  Daily Customers: {:>3}  │  Rent: ${:>6.0}   ║",
        store.employees.len(),
        store.effective_customers(),
        store.daily_rent
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    if store.inventory.is_empty() {
        println!("║  (No inventory yet - buy some products!)                     ║");
    } else {
        println!(
            "║  {:20} {:>8} {:>12} {:>12}      ║",
            "Product", "Qty", "Retail $", "Markup %"
        );
        println!("║  {:─<20} {:─>8} {:─>12} {:─>12}      ║", "", "", "", "");

        for (product_id, item) in &store.inventory {
            if let Some(product) = game.get_product(*product_id) {
                let wholesale = game.market.get_wholesale_price(*product_id).unwrap_or(0.0);
                let markup = Market::calculate_markup(wholesale, item.retail_price);
                println!(
                    "║  {:20} {:>8} {:>12.2} {:>11.1}%      ║",
                    product.name, item.quantity, item.retail_price, markup
                );
            }
        }
    }

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Total Items: {:>6}  │  Inventory Value: ${:>10.2}       ║",
        store.total_items(),
        store.total_inventory_value()
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    wait_for_enter();
}

/// Displays available products for purchase (retail goods only, no raw materials)
pub fn display_buy_menu(game: &GameState) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  WHOLESALE MARKET                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  {:3} {:20} {:>12} {:>15}        ║", "ID", "Product", "Price", "Category");
    println!("║  {:─<3} {:─<20} {:─>12} {:─>15}        ║", "", "", "", "");

    // Only show products that can be sold retail (not raw materials)
    for product in &game.products {
        if !product.product_type.can_sell_retail() {
            continue;
        }
        let wholesale = game.market.get_wholesale_price(product.id).unwrap_or(product.base_price);
        println!(
            "║  {:>3} {:20} ${:>10.2} {:>15}        ║",
            product.id,
            product.name,
            wholesale,
            product.category.name()
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

/// Represents an item in the shopping cart
struct CartItem {
    product_id: u32,
    product_name: String,
    quantity: u32,
    unit_price: f64,
}

impl CartItem {
    fn total(&self) -> f64 {
        self.unit_price * self.quantity as f64
    }
}

/// Displays the shopping cart
fn display_cart(cart: &[CartItem], player_cash: f64) {
    if cart.is_empty() {
        println!("  Cart is empty.");
    } else {
        println!("  {:3} {:20} {:>6} {:>10} {:>12}", "#", "Product", "Qty", "Unit $", "Subtotal");
        println!("  {:─<3} {:─<20} {:─>6} {:─>10} {:─>12}", "", "", "", "", "");
        for (idx, item) in cart.iter().enumerate() {
            println!(
                "  {:>3} {:20} {:>6} {:>10.2} {:>12.2}",
                idx + 1,
                item.product_name,
                item.quantity,
                item.unit_price,
                item.total()
            );
        }
        let cart_total: f64 = cart.iter().map(|i| i.total()).sum();
        println!("  {:─<3} {:─<20} {:─>6} {:─>10} {:─>12}", "", "", "", "", "");
        println!("  {:24} {:>6} {:>10} ${:>11.2}", "TOTAL", "", "", cart_total);
        println!();
        let remaining = player_cash - cart_total;
        if remaining >= 0.0 {
            println!("  After purchase: ${:.2}", remaining);
        } else {
            println!("  WARNING: ${:.2} over budget!", -remaining);
        }
    }
}

/// Handles buying inventory with cart system
pub fn handle_buy_inventory(game: &mut GameState) {
    let mut cart: Vec<CartItem> = Vec::new();

    loop {
        clear_screen();
        display_buy_menu(game);

        println!("Your cash: ${:.2}", game.player.cash);
        println!();

        // Display cart
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    SHOPPING CART                             ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        display_cart(&cart, game.player.cash);
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [A] Add item    [R] Remove item    [C] Checkout    [0] Cancel║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ").to_lowercase();

        match input.trim() {
            "0" => return,
            "a" => {
                // Add item to cart
                let product_id = match read_number("Enter product ID: ") {
                    Some(id) => id,
                    None => {
                        println!("Invalid product ID.");
                        wait_for_enter();
                        continue;
                    }
                };

                let product = match game.get_product(product_id) {
                    Some(p) => p.clone(),
                    None => {
                        println!("Product not found.");
                        wait_for_enter();
                        continue;
                    }
                };

                let quantity = match read_number("Enter quantity: ") {
                    Some(0) => continue,
                    Some(q) => q,
                    None => {
                        println!("Invalid quantity.");
                        wait_for_enter();
                        continue;
                    }
                };

                let unit_price = game
                    .market
                    .get_wholesale_price(product_id)
                    .unwrap_or(product.base_price);

                // Check if product already in cart, if so add to quantity
                if let Some(existing) = cart.iter_mut().find(|i| i.product_id == product_id) {
                    existing.quantity += quantity;
                    println!("Updated {} quantity to {}", product.name, existing.quantity);
                } else {
                    cart.push(CartItem {
                        product_id,
                        product_name: product.name.clone(),
                        quantity,
                        unit_price,
                    });
                    println!("Added {} x {} to cart", quantity, product.name);
                }
                wait_for_enter();
            }
            "r" => {
                // Remove item from cart
                if cart.is_empty() {
                    println!("Cart is empty.");
                    wait_for_enter();
                    continue;
                }

                let item_num = match read_number("Enter item # to remove (0 to cancel): ") {
                    Some(0) => continue,
                    Some(n) if n > 0 && (n as usize) <= cart.len() => n as usize - 1,
                    _ => {
                        println!("Invalid item number.");
                        wait_for_enter();
                        continue;
                    }
                };

                let removed = cart.remove(item_num);
                println!("Removed {} from cart", removed.product_name);
                wait_for_enter();
            }
            "c" => {
                // Checkout
                if cart.is_empty() {
                    println!("Cart is empty. Add items first!");
                    wait_for_enter();
                    continue;
                }

                let cart_total: f64 = cart.iter().map(|i| i.total()).sum();

                if cart_total > game.player.cash {
                    println!(
                        "Not enough cash! Need ${:.2}, have ${:.2}",
                        cart_total, game.player.cash
                    );
                    wait_for_enter();
                    continue;
                }

                // Confirm purchase
                println!();
                println!("Confirm purchase of {} items for ${:.2}?", cart.len(), cart_total);
                let confirm = read_input("[Y/n]: ");
                if confirm.to_lowercase() == "n" {
                    continue;
                }

                // Process all purchases
                let mut success_count = 0;
                let mut total_spent = 0.0;

                for item in &cart {
                    match game.buy_inventory(item.product_id, item.quantity) {
                        Ok(cost) => {
                            success_count += 1;
                            total_spent += cost;
                        }
                        Err(e) => {
                            println!("Failed to buy {}: {}", item.product_name, e);
                        }
                    }
                }

                println!();
                println!("═══════════════════════════════════════════════════════════════");
                println!("  PURCHASE COMPLETE!");
                println!("  Bought {} item types for ${:.2}", success_count, total_spent);
                println!("  Remaining cash: ${:.2}", game.player.cash);
                println!("═══════════════════════════════════════════════════════════════");
                wait_for_enter();
                return;
            }
            _ => {
                // Try to parse as product ID for quick add
                if let Ok(product_id) = input.trim().parse::<u32>() {
                    if let Some(product) = game.get_product(product_id) {
                        let product = product.clone();
                        let quantity = match read_number("Enter quantity: ") {
                            Some(0) => continue,
                            Some(q) => q,
                            None => {
                                println!("Invalid quantity.");
                                wait_for_enter();
                                continue;
                            }
                        };

                        let unit_price = game
                            .market
                            .get_wholesale_price(product_id)
                            .unwrap_or(product.base_price);

                        if let Some(existing) = cart.iter_mut().find(|i| i.product_id == product_id)
                        {
                            existing.quantity += quantity;
                        } else {
                            cart.push(CartItem {
                                product_id,
                                product_name: product.name.clone(),
                                quantity,
                                unit_price,
                            });
                        }
                        println!("Added {} x {} to cart", quantity, product.name);
                        wait_for_enter();
                    } else {
                        println!("Invalid choice or product ID.");
                        wait_for_enter();
                    }
                } else {
                    println!("Invalid choice. Use A/R/C/0 or enter a product ID.");
                    wait_for_enter();
                }
            }
        }
    }
}

/// Handles setting retail prices - loops until user chooses to exit
pub fn handle_set_prices(game: &mut GameState) {
    loop {
        clear_screen();
        let store = game.current_store();

        if store.inventory.is_empty() {
            println!("You have no inventory to price. Buy some products first!");
            wait_for_enter();
            return;
        }

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    SET RETAIL PRICES                         ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  {:3} {:20} {:>10} {:>10} {:>10}   ║",
            "ID", "Product", "Wholesale", "Current", "Markup"
        );
        println!("║  {:─<3} {:─<20} {:─>10} {:─>10} {:─>10}   ║", "", "", "", "", "");

        for (product_id, item) in &store.inventory {
            if let Some(product) = game.get_product(*product_id) {
                let wholesale = game.market.get_wholesale_price(*product_id).unwrap_or(0.0);
                let markup = Market::calculate_markup(wholesale, item.retail_price);
                println!(
                    "║  {:>3} {:20} ${:>8.2} ${:>8.2} {:>8.1}%   ║",
                    product.id, product.name, wholesale, item.retail_price, markup
                );
            }
        }
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let product_id = match read_number("Enter product ID to reprice (0 to return to menu): ") {
            Some(0) => return,
            Some(id) => id,
            None => {
                println!("Invalid product ID.");
                continue;
            }
        };

        let wholesale = game.market.get_wholesale_price(product_id).unwrap_or(0.0);
        if wholesale == 0.0 {
            println!("Product not in inventory.");
            continue;
        }

        println!("Wholesale price: ${:.2}", wholesale);
        println!("Suggested markups: 25%=${:.2}, 50%=${:.2}, 100%=${:.2}",
            Market::suggest_retail_price(wholesale, 25.0),
            Market::suggest_retail_price(wholesale, 50.0),
            Market::suggest_retail_price(wholesale, 100.0)
        );

        let new_price = match read_float("Enter new retail price: $") {
            Some(p) if p > 0.0 => p,
            _ => {
                println!("Invalid price.");
                continue;
            }
        };

        match game.set_retail_price(product_id, new_price) {
            Ok(()) => {
                let markup = Market::calculate_markup(wholesale, new_price);
                println!("Price updated! New markup: {:.1}%", markup);
            }
            Err(e) => println!("ERROR: {}", e),
        }

        println!();
        let choice = read_input("Set another price? [Y/n]: ");
        if choice.to_lowercase() == "n" {
            return;
        }
    }
}

/// Displays the results of advancing a day
pub fn display_day_result(result: &DayResult, new_day: u32, game: &GameState) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║                     DAY {} RESULTS                            ║",
        new_day - 1
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Economic state section
    println!(
        "║  ECONOMY: {:12} (Sales {:>3}%, Prices {:>3}%)             ║",
        result.economic_state.name(),
        (result.economic_state.sales_multiplier() * 100.0) as i32,
        (result.economic_state.price_multiplier() * 100.0) as i32
    );
    if let Some(ref change) = result.economic_change {
        println!("║    >>> {} <<<                           ║", change);
    }

    // Sales section
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  SALES:                                                      ║");
    if result.sales_by_product.is_empty() {
        println!("║    No sales today. Check your prices or stock!               ║");
    } else {
        for (name, qty, revenue) in &result.sales_by_product {
            println!(
                "║    Sold {:>3} x {:18} = ${:>10.2}          ║",
                qty, name, revenue
            );
        }
    }
    println!(
        "║    Total Revenue: ${:>10.2}                                ║",
        result.total_revenue
    );

    // Production section (if any factories)
    if !result.production_completed.is_empty() {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  PRODUCTION COMPLETED:                                       ║");
        for prod in &result.production_completed {
            let product_name = game
                .get_product(prod.product_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown");
            println!(
                "║    {} x {} ({})                              ║",
                prod.quantity, product_name, prod.recipe_name
            );
        }
    }

    // Expenses section
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  EXPENSES:                                                   ║");

    // Store expenses
    for (store_name, rent, salaries) in &result.expenses_by_store {
        let total = rent + salaries;
        println!(
            "║    Store {}: ${:.0}                                      ║",
            store_name, total
        );
    }

    // Factory expenses
    for (factory_name, rent, salaries) in &result.expenses_by_factory {
        let total = rent + salaries;
        println!(
            "║    Factory {}: ${:.0}                                    ║",
            factory_name, total
        );
    }

    println!(
        "║    Total Expenses: ${:>10.2}                               ║",
        result.total_expenses
    );

    // Loan section (if there are any loan-related events)
    let has_loan_events = result.loan_interest_accrued > 0.01
        || !result.loan_payments.is_empty()
        || !result.loans_due.is_empty()
        || !result.loans_due_soon.is_empty()
        || result.term_loan_penalties > 0.01;

    if has_loan_events {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  LOANS:                                                      ║");

        if result.loan_interest_accrued > 0.01 {
            println!(
                "║    Interest accrued: ${:>10.2}                             ║",
                result.loan_interest_accrued
            );
        }

        for (loan_id, amount) in &result.loan_payments {
            println!(
                "║    Auto-payment (Loan #{}): ${:>10.2}                      ║",
                loan_id, amount
            );
        }

        for (loan_id, amount) in &result.loans_due {
            println!(
                "║    TERM LOAN #{} DUE: ${:>10.2}                            ║",
                loan_id, amount
            );
        }

        if result.term_loan_penalties > 0.01 {
            println!(
                "║    DEFAULT PENALTY: ${:>10.2}                              ║",
                result.term_loan_penalties
            );
        }

        // Warnings for upcoming due loans
        for (loan_id, days, balance) in &result.loans_due_soon {
            println!(
                "║    WARNING: Loan #{} due in {} day(s)! (${:.2})           ║",
                loan_id, days, balance
            );
        }
    }

    // Auto-transfers section
    if !result.auto_transfers.is_empty() {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  AUTO-TRANSFERS (Supply Chain):                              ║");
        for (factory, store, product, qty) in &result.auto_transfers {
            println!(
                "║    {} -> {}: {} x {}           ║",
                factory, store, qty, product
            );
        }
    }

    // Market & Competitors section
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  MARKET: Your share: {:>5.1}%                                  ║",
        result.player_market_share * 100.0
    );

    // Competitor events
    if !result.competitor_events.is_empty() {
        println!("║  COMPETITOR NEWS:                                            ║");
        for event in &result.competitor_events {
            println!("║    >>> {}                    ║", event);
        }
    }

    // Stock market section (if player has holdings or significant price moves)
    let significant_moves: Vec<_> = result.stock_changes.iter()
        .filter(|(_, old, new)| {
            let change_pct = ((new - old) / old * 100.0).abs();
            change_pct > 3.0  // Only show moves > 3%
        })
        .collect();

    if !significant_moves.is_empty() || result.dividends_earned > 0.01 {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  STOCK MARKET:                                               ║");

        for (symbol, old, new) in &significant_moves {
            let change = new - old;
            let pct = (change / old) * 100.0;
            let arrow = if change > 0.0 { "▲" } else { "▼" };
            println!(
                "║    {} ${:.2} {} {:.1}%                                      ║",
                symbol, new, arrow, pct.abs()
            );
        }

        if result.dividends_earned > 0.01 {
            println!(
                "║    Dividends earned: ${:.2}                                  ║",
                result.dividends_earned
            );
        }
    }

    // Net profit section
    println!("╠══════════════════════════════════════════════════════════════╣");
    let profit_label = if result.net_profit >= 0.0 {
        "NET PROFIT"
    } else {
        "NET LOSS"
    };
    println!(
        "║  {}: ${:>10.2}                                      ║",
        profit_label,
        result.net_profit.abs()
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    wait_for_enter();
}

/// Reads a line of input from the user
pub fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Reads a number from input
pub fn read_number(prompt: &str) -> Option<u32> {
    let input = read_input(prompt);
    input.parse().ok()
}

/// Reads a floating point number from input
pub fn read_float(prompt: &str) -> Option<f64> {
    let input = read_input(prompt);
    input.parse().ok()
}

/// Waits for the user to press Enter
pub fn wait_for_enter() {
    read_input("Press Enter to continue...");
}

/// Displays a welcome message
pub fn display_welcome() {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║              WELCOME TO BUSINESS TYCOON                      ║");
    println!("║                   Rust Edition v0.1                          ║");
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  You are starting your journey as a retail entrepreneur!    ║");
    println!("║                                                              ║");
    println!("║  Your goal: Buy products wholesale, sell them retail,       ║");
    println!("║  and grow your business empire!                             ║");
    println!("║                                                              ║");
    println!("║  Starting capital: $1,000                                   ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    wait_for_enter();
}

/// Handles store management submenu
pub fn handle_manage_stores(game: &mut GameState) {
    loop {
        clear_screen();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    MANAGE STORES                             ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Your cash: ${:>10.2}                                      ║", game.player.cash);
        println!("╠══════════════════════════════════════════════════════════════╣");

        // Display all stores
        for (idx, store) in game.player.stores.iter().enumerate() {
            let current_marker = if idx == game.current_store { "→" } else { " " };
            println!(
                "║ {} [{}] {:20} │ Items: {:>4} │ Staff: {}        ║",
                current_marker,
                idx + 1,
                store.name,
                store.total_items(),
                store.employees.len()
            );
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [1] View all stores                                         ║");
        println!("║  [2] Switch active store                                     ║");
        println!("║  [3] Buy new store ($5,000)                                  ║");
        println!("║  [0] Back to main menu                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ");
        match input.trim() {
            "0" => return,
            "1" => {
                display_all_stores(game);
            }
            "2" => {
                handle_switch_store(game);
            }
            "3" => {
                handle_buy_new_store(game);
            }
            _ => println!("Invalid choice."),
        }
    }
}

/// Displays detailed info about all stores
fn display_all_stores(game: &GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     ALL STORES                               ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    for (idx, store) in game.player.stores.iter().enumerate() {
        let current_marker = if idx == game.current_store {
            "(ACTIVE)"
        } else {
            ""
        };
        println!(
            "║  Store #{}: {} {}",
            store.id, store.name, current_marker
        );
        println!(
            "║    Inventory: {} items (${:.2} value)",
            store.total_items(),
            store.total_inventory_value()
        );
        println!(
            "║    Employees: {} │ Daily Customers: {}",
            store.employees.len(),
            store.effective_customers()
        );
        println!(
            "║    Daily Expenses: ${:.2} (Rent: ${:.0}, Salaries: ${:.0})",
            store.daily_expenses(),
            store.daily_rent,
            store.employees.iter().map(|e| e.salary).sum::<f64>()
        );
        println!("║  ──────────────────────────────────────────────────────────  ║");
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    wait_for_enter();
}

/// Handles switching between stores
fn handle_switch_store(game: &mut GameState) {
    if game.player.stores.len() == 1 {
        println!("You only have one store. Buy more stores first!");
        wait_for_enter();
        return;
    }

    println!("Available stores:");
    for (idx, store) in game.player.stores.iter().enumerate() {
        let current_marker = if idx == game.current_store { " (current)" } else { "" };
        println!("  [{}] {}{}", idx + 1, store.name, current_marker);
    }

    let store_num = match read_number("Enter store number (0 to cancel): ") {
        Some(0) => return,
        Some(n) if n > 0 && (n as usize) <= game.player.stores.len() => n as usize - 1,
        _ => {
            println!("Invalid store number.");
            wait_for_enter();
            return;
        }
    };

    if game.switch_store(store_num).is_ok() {
        println!(
            "Switched to: {}",
            game.player.stores[store_num].name
        );
    }
    wait_for_enter();
}

/// Handles buying a new store
fn handle_buy_new_store(game: &mut GameState) {
    println!("Buy a new store for $5,000");
    println!("Your cash: ${:.2}", game.player.cash);
    println!();

    if game.player.cash < 5000.0 {
        println!("Not enough cash! You need $5,000.");
        wait_for_enter();
        return;
    }

    let name = read_input("Enter name for new store (or 0 to cancel): ");
    if name == "0" || name.is_empty() {
        return;
    }

    match game.buy_new_store(&name) {
        Ok(reactions) => {
            println!();
            println!("SUCCESS! Purchased new store: {}", name);
            println!("Remaining cash: ${:.2}", game.player.cash);
            // Show competitor reactions
            if !reactions.is_empty() {
                println!();
                println!("COMPETITOR REACTIONS:");
                for reaction in reactions {
                    println!("  >>> {}", reaction);
                }
            }
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

/// Handles staff management submenu
pub fn handle_manage_staff(game: &mut GameState) {
    loop {
        clear_screen();
        let store = game.current_store();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    MANAGE STAFF                              ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Store: {:20}  │  Cash: ${:>10.2}   ║",
            store.name, game.player.cash
        );
        println!("╠══════════════════════════════════════════════════════════════╣");

        if store.employees.is_empty() {
            println!("║  No employees yet.                                           ║");
        } else {
            println!("║  Current Employees:                                          ║");
            for (idx, emp) in store.employees.iter().enumerate() {
                println!(
                    "║    [{}] {:30} ${:.0}/day          ║",
                    idx + 1,
                    emp.name,
                    emp.salary
                );
            }
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Daily Customers: {:>3} (base: 50, +20% per employee)        ║",
            store.effective_customers()
        );
        println!(
            "║  Total Daily Salaries: ${:>6.0}                               ║",
            store.employees.iter().map(|e| e.salary).sum::<f64>()
        );
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [1] View employees                                          ║");
        println!("║  [2] Hire employee ($50/day)                                 ║");
        println!("║  [3] Fire employee                                           ║");
        println!("║  [0] Back to main menu                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ");
        match input.trim() {
            "0" => return,
            "1" => {
                display_employees(game);
            }
            "2" => {
                handle_hire_employee(game);
            }
            "3" => {
                handle_fire_employee(game);
            }
            _ => println!("Invalid choice."),
        }
    }
}

/// Displays employees of the current store
fn display_employees(game: &GameState) {
    clear_screen();
    let store = game.current_store();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 {} - EMPLOYEES                 ║", store.name);
    println!("╠══════════════════════════════════════════════════════════════╣");

    if store.employees.is_empty() {
        println!("║  No employees hired yet.                                     ║");
        println!("║  Hire employees to increase customer traffic!                ║");
    } else {
        for (idx, emp) in store.employees.iter().enumerate() {
            println!(
                "║  {}. {:40} ${:.0}/day    ║",
                idx + 1,
                emp.name,
                emp.salary
            );
        }
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Customer Bonus: +{}% ({} → {} customers/day)             ║",
            store.employees.len() * 20,
            store.daily_customers,
            store.effective_customers()
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    wait_for_enter();
}

/// Handles hiring a new employee
fn handle_hire_employee(game: &mut GameState) {
    let store = game.current_store();

    if store.employees.len() >= 3 {
        println!("Maximum of 3 employees per store reached!");
        wait_for_enter();
        return;
    }

    println!("Hire a new employee ($50/day salary)");
    println!(
        "Current employees: {}/3",
        store.employees.len()
    );
    println!();

    let name = read_input("Enter employee name (or 0 to cancel): ");
    if name == "0" || name.is_empty() {
        return;
    }

    match game.current_store_mut().hire_employee(&name) {
        Ok(()) => {
            println!();
            println!("SUCCESS! Hired: {}", name);
            println!(
                "New customer count: {} (+20%)",
                game.current_store().effective_customers()
            );
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

/// Handles firing an employee
fn handle_fire_employee(game: &mut GameState) {
    let store = game.current_store();

    if store.employees.is_empty() {
        println!("No employees to fire!");
        wait_for_enter();
        return;
    }

    println!("Fire an employee:");
    for (idx, emp) in store.employees.iter().enumerate() {
        println!("  [{}] {} (${:.0}/day)", idx + 1, emp.name, emp.salary);
    }
    println!();

    let emp_num = match read_number("Enter employee number to fire (0 to cancel): ") {
        Some(0) => return,
        Some(n) if n > 0 && (n as usize) <= store.employees.len() => n as usize - 1,
        _ => {
            println!("Invalid employee number.");
            wait_for_enter();
            return;
        }
    };

    match game.current_store_mut().fire_employee(emp_num) {
        Ok(fired) => {
            println!();
            println!("Fired: {}", fired.name);
            println!(
                "New customer count: {}",
                game.current_store().effective_customers()
            );
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

// ==================== FACTORY MANAGEMENT ====================

/// Handles factory management submenu
pub fn handle_manage_factories(game: &mut GameState) {
    loop {
        clear_screen();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    MANAGE FACTORIES                          ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Your cash: ${:>10.2}                                      ║",
            game.player.cash
        );
        println!("╠══════════════════════════════════════════════════════════════╣");

        if game.player.factories.is_empty() {
            println!("║  No factories yet. Buy one to start manufacturing!          ║");
        } else {
            // Display all factories
            for (idx, factory) in game.player.factories.iter().enumerate() {
                let current_marker = if Some(idx) == game.current_factory {
                    "→"
                } else {
                    " "
                };
                println!(
                    "║ {} [{}] {:20} │ Jobs: {}/{} │ Workers: {}   ║",
                    current_marker,
                    idx + 1,
                    factory.name,
                    factory.active_jobs(),
                    factory.production_slots(),
                    factory.workers.len()
                );
            }
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [1] View factory status                                     ║");
        println!("║  [2] Buy raw materials                                       ║");
        println!("║  [3] Start production                                        ║");
        println!("║  [4] Transfer goods to store                                 ║");
        println!("║  [5] Manage factory workers                                  ║");
        println!("║  [6] Manage supply chain                                     ║");
        println!("║  [7] Switch factory                                          ║");
        println!("║  [8] Buy new factory ($10,000)                               ║");
        println!("║  [0] Back to main menu                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ");
        match input.trim() {
            "0" => return,
            "1" => display_factory_status(game),
            "2" => handle_buy_raw_materials(game),
            "3" => handle_start_production(game),
            "4" => handle_transfer_goods(game),
            "5" => handle_factory_workers(game),
            "6" => handle_supply_chain(game),
            "7" => handle_switch_factory(game),
            "8" => handle_buy_new_factory(game),
            _ => println!("Invalid choice."),
        }
    }
}

/// Displays detailed factory status
fn display_factory_status(game: &GameState) {
    clear_screen();

    if game.current_factory.is_none() {
        println!("No factory selected. Buy or select a factory first!");
        wait_for_enter();
        return;
    }

    let factory = game.current_factory().unwrap();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║  {:^58}  ║",
        format!("{} - Status", factory.name)
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Workers: {}/3  │  Production Slots: {}/{}                   ║",
        factory.workers.len(),
        factory.active_jobs(),
        factory.production_slots()
    );
    println!(
        "║  Daily Expenses: ${:.0} (Rent: ${:.0}, Salaries: ${:.0})      ║",
        factory.daily_expenses(),
        factory.daily_rent,
        factory.workers.iter().map(|w| w.salary).sum::<f64>()
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Raw materials
    println!("║  RAW MATERIALS:                                              ║");
    if factory.raw_materials.is_empty() {
        println!("║    (None)                                                    ║");
    } else {
        for (product_id, quantity) in &factory.raw_materials {
            if *quantity > 0 {
                let name = game
                    .get_product(*product_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("Unknown");
                println!("║    {:30} x {:>6}                   ║", name, quantity);
            }
        }
    }

    // Production queue
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  PRODUCTION IN PROGRESS:                                     ║");
    if factory.production_queue.is_empty() {
        println!("║    (None)                                                    ║");
    } else {
        for job in &factory.production_queue {
            let product_name = game
                .get_product(job.output_product_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown");
            println!(
                "║    {} → {} ({} day(s) left)                        ║",
                job.recipe_name, product_name, job.days_remaining
            );
        }
    }

    // Finished goods
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  FINISHED GOODS (ready to transfer):                        ║");
    if factory.finished_goods.is_empty() || factory.total_finished_goods() == 0 {
        println!("║    (None)                                                    ║");
    } else {
        for (product_id, quantity) in &factory.finished_goods {
            if *quantity > 0 {
                let name = game
                    .get_product(*product_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("Unknown");
                println!("║    {:30} x {:>6}                   ║", name, quantity);
            }
        }
    }

    // Supply chain
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  SUPPLY CHAIN: Auto-transfer {}                              ║",
        if factory.auto_transfer { "ON " } else { "OFF" }
    );
    if factory.connected_stores.is_empty() {
        println!("║    (Not connected to any stores)                             ║");
    } else {
        for store_id in &factory.connected_stores {
            if let Some(store_name) = game.get_store_name_by_id(*store_id) {
                let is_primary = factory.primary_store() == Some(*store_id);
                let marker = if is_primary { " [PRIMARY]" } else { "" };
                println!("║    → {}{}                                    ║", store_name, marker);
            }
        }
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    wait_for_enter();
}

/// Handles buying raw materials for the factory
fn handle_buy_raw_materials(game: &mut GameState) {
    if game.current_factory.is_none() {
        println!("No factory selected. Buy or select a factory first!");
        wait_for_enter();
        return;
    }

    let mut cart: Vec<CartItem> = Vec::new();

    loop {
        clear_screen();

        // Display raw materials market
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                  RAW MATERIALS MARKET                        ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  {:3} {:25} {:>12}                   ║",
            "ID", "Material", "Price"
        );
        println!("║  {:─<3} {:─<25} {:─>12}                   ║", "", "", "");

        for product in Product::raw_materials() {
            let wholesale = game
                .market
                .get_wholesale_price(product.id)
                .unwrap_or(product.base_price);
            println!(
                "║  {:>3} {:25} ${:>10.2}                   ║",
                product.id, product.name, wholesale
            );
        }
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        println!("Your cash: ${:.2}", game.player.cash);
        println!();

        // Display cart
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    SHOPPING CART                             ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        display_cart(&cart, game.player.cash);
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [A] Add item    [R] Remove item    [C] Checkout    [0] Cancel║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ").to_lowercase();

        match input.trim() {
            "0" => return,
            "a" => {
                let product_id = match read_number("Enter material ID: ") {
                    Some(id) => id,
                    None => {
                        println!("Invalid ID.");
                        wait_for_enter();
                        continue;
                    }
                };

                let product = match game.get_product(product_id) {
                    Some(p) if p.product_type.is_raw_material() => p.clone(),
                    _ => {
                        println!("Not a valid raw material.");
                        wait_for_enter();
                        continue;
                    }
                };

                let quantity = match read_number("Enter quantity: ") {
                    Some(0) => continue,
                    Some(q) => q,
                    None => {
                        println!("Invalid quantity.");
                        wait_for_enter();
                        continue;
                    }
                };

                let unit_price = game
                    .market
                    .get_wholesale_price(product_id)
                    .unwrap_or(product.base_price);

                if let Some(existing) = cart.iter_mut().find(|i| i.product_id == product_id) {
                    existing.quantity += quantity;
                } else {
                    cart.push(CartItem {
                        product_id,
                        product_name: product.name.clone(),
                        quantity,
                        unit_price,
                    });
                }
                println!("Added {} x {} to cart", quantity, product.name);
                wait_for_enter();
            }
            "r" => {
                if cart.is_empty() {
                    println!("Cart is empty.");
                    wait_for_enter();
                    continue;
                }

                let item_num = match read_number("Enter item # to remove (0 to cancel): ") {
                    Some(0) => continue,
                    Some(n) if n > 0 && (n as usize) <= cart.len() => n as usize - 1,
                    _ => {
                        println!("Invalid item number.");
                        wait_for_enter();
                        continue;
                    }
                };

                let removed = cart.remove(item_num);
                println!("Removed {} from cart", removed.product_name);
                wait_for_enter();
            }
            "c" => {
                if cart.is_empty() {
                    println!("Cart is empty. Add items first!");
                    wait_for_enter();
                    continue;
                }

                let cart_total: f64 = cart.iter().map(|i| i.total()).sum();

                if cart_total > game.player.cash {
                    println!(
                        "Not enough cash! Need ${:.2}, have ${:.2}",
                        cart_total, game.player.cash
                    );
                    wait_for_enter();
                    continue;
                }

                // Confirm purchase
                println!();
                println!(
                    "Confirm purchase of {} items for ${:.2}?",
                    cart.len(),
                    cart_total
                );
                let confirm = read_input("[Y/n]: ");
                if confirm.to_lowercase() == "n" {
                    continue;
                }

                // Process all purchases
                let mut success_count = 0;
                let mut total_spent = 0.0;

                for item in &cart {
                    match game.buy_raw_materials(item.product_id, item.quantity) {
                        Ok(cost) => {
                            success_count += 1;
                            total_spent += cost;
                        }
                        Err(e) => {
                            println!("Failed to buy {}: {}", item.product_name, e);
                        }
                    }
                }

                println!();
                println!("═══════════════════════════════════════════════════════════════");
                println!("  PURCHASE COMPLETE!");
                println!(
                    "  Bought {} material types for ${:.2}",
                    success_count, total_spent
                );
                println!("  Remaining cash: ${:.2}", game.player.cash);
                println!("═══════════════════════════════════════════════════════════════");
                wait_for_enter();
                return;
            }
            _ => {
                // Try to parse as material ID for quick add
                if let Ok(product_id) = input.trim().parse::<u32>() {
                    if let Some(product) = game.get_product(product_id) {
                        if product.product_type.is_raw_material() {
                            let product = product.clone();
                            let quantity = match read_number("Enter quantity: ") {
                                Some(0) => continue,
                                Some(q) => q,
                                None => {
                                    println!("Invalid quantity.");
                                    wait_for_enter();
                                    continue;
                                }
                            };

                            let unit_price = game
                                .market
                                .get_wholesale_price(product_id)
                                .unwrap_or(product.base_price);

                            if let Some(existing) =
                                cart.iter_mut().find(|i| i.product_id == product_id)
                            {
                                existing.quantity += quantity;
                            } else {
                                cart.push(CartItem {
                                    product_id,
                                    product_name: product.name.clone(),
                                    quantity,
                                    unit_price,
                                });
                            }
                            println!("Added {} x {} to cart", quantity, product.name);
                            wait_for_enter();
                        } else {
                            println!("Not a raw material.");
                            wait_for_enter();
                        }
                    } else {
                        println!("Invalid choice or material ID.");
                        wait_for_enter();
                    }
                } else {
                    println!("Invalid choice. Use A/R/C/0 or enter a material ID.");
                    wait_for_enter();
                }
            }
        }
    }
}

/// Handles starting production
fn handle_start_production(game: &mut GameState) {
    if game.current_factory.is_none() {
        println!("No factory selected. Buy or select a factory first!");
        wait_for_enter();
        return;
    }

    let factory = game.current_factory().unwrap();

    if factory.available_slots() == 0 {
        println!("No available production slots! Wait for current jobs to complete.");
        wait_for_enter();
        return;
    }

    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    START PRODUCTION                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Available slots: {}/{}                                       ║",
        factory.available_slots(),
        factory.production_slots()
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:2} {:20} {:>5} {:>8} {:>6}              ║",
        "ID", "Recipe", "Days", "Cost", "Max"
    );
    println!("║  {:─<2} {:─<20} {:─>5} {:─>8} {:─>6}              ║", "", "", "", "", "");

    for recipe in &game.recipes {
        let material_cost = recipe.material_cost(|id| {
            game.market.get_wholesale_price(id).unwrap_or(0.0)
        });
        let max_producible = factory.max_producible(recipe);

        println!(
            "║  {:>2} {:20} {:>3} d ${:>7.0} {:>6}              ║",
            recipe.id, recipe.name, recipe.production_days, material_cost, max_producible
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Show current raw materials
    println!("Your raw materials:");
    let factory = game.current_factory().unwrap();
    let mut has_materials = false;
    for (product_id, quantity) in &factory.raw_materials {
        if *quantity > 0 {
            has_materials = true;
            let name = game
                .get_product(*product_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown");
            println!("  {} x {}", quantity, name);
        }
    }
    if !has_materials {
        println!("  (None - buy raw materials first!)");
    }
    println!();

    let recipe_id = match read_number("Enter recipe ID to produce (0 to cancel): ") {
        Some(0) => return,
        Some(id) => id,
        None => {
            println!("Invalid recipe ID.");
            wait_for_enter();
            return;
        }
    };

    let recipe = match game.get_recipe(recipe_id) {
        Some(r) => r.clone(),
        None => {
            println!("Recipe not found.");
            wait_for_enter();
            return;
        }
    };

    let max_producible = game.max_producible(recipe_id).unwrap_or(0);

    // Show recipe details
    println!();
    println!("Recipe: {}", recipe.name);
    println!("Requires (per batch):");
    for ing in &recipe.ingredients {
        let name = game
            .get_product(ing.product_id)
            .map(|p| p.name.as_str())
            .unwrap_or("Unknown");
        let have = game.current_factory().unwrap().get_raw_material(ing.product_id);
        let batches = if ing.quantity > 0 { have / ing.quantity } else { 0 };
        println!("  {} x {} (have: {}, enough for {} batches)", ing.quantity, name, have, batches);
    }
    println!("Production time: {} day(s) per batch", recipe.production_days);
    println!("Max producible now: {} (limited by slots and materials)", max_producible);
    println!();

    if max_producible == 0 {
        println!("Cannot produce this recipe - check slots and materials!");
        wait_for_enter();
        return;
    }

    // Ask for quantity
    let quantity = if max_producible == 1 {
        // Only 1 possible, just confirm
        let confirm = read_input("Start 1 batch? [Y/n]: ");
        if confirm.to_lowercase() == "n" {
            return;
        }
        1
    } else {
        // Multiple possible, ask for quantity
        println!("How many batches to produce? (1-{}, or 'all' for max)", max_producible);
        let input = read_input("Quantity: ");

        if input.to_lowercase() == "all" {
            max_producible
        } else {
            match input.parse::<u32>() {
                Ok(0) => return,
                Ok(q) if q <= max_producible => q,
                Ok(q) => {
                    println!("Reducing to maximum: {}", max_producible);
                    wait_for_enter();
                    max_producible.min(q)
                }
                Err(_) => {
                    println!("Invalid quantity.");
                    wait_for_enter();
                    return;
                }
            }
        }
    };

    match game.start_production_batch(recipe_id, quantity) {
        Ok(started) => {
            let output_name = game
                .get_product(recipe.output_product_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown");
            println!();
            println!("Production started!");
            println!(
                "Queued {} batch(es) - will produce {} x {} each in {} day(s)",
                started, recipe.output_quantity, output_name, recipe.production_days
            );
            println!(
                "Total output: {} x {}",
                started * recipe.output_quantity,
                output_name
            );
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

/// Handles transferring goods from factory to store
fn handle_transfer_goods(game: &mut GameState) {
    if game.current_factory.is_none() {
        println!("No factory selected. Buy or select a factory first!");
        wait_for_enter();
        return;
    }

    let factory = game.current_factory().unwrap();

    if factory.total_finished_goods() == 0 {
        println!("No finished goods to transfer.");
        wait_for_enter();
        return;
    }

    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  TRANSFER TO STORE                           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Finished goods available:                                   ║");

    let mut available_goods: Vec<(u32, u32, String)> = Vec::new();
    for (product_id, quantity) in &factory.finished_goods {
        if *quantity > 0 {
            let name = game
                .get_product(*product_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            println!("║    ID {:>2}: {:25} x {:>6}            ║", product_id, name, quantity);
            available_goods.push((*product_id, *quantity, name));
        }
    }

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Your stores (must be connected via supply chain):           ║");
    let factory = game.current_factory().unwrap();
    for (idx, store) in game.player.stores.iter().enumerate() {
        let connected = factory.is_connected_to(store.id);
        let status = if connected { "[OK]" } else { "[NOT CONNECTED]" };
        println!("║    [{}] {:30} {}       ║", idx + 1, store.name, status);
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Check if factory has any connections
    if factory.connected_stores.is_empty() {
        println!("This factory is not connected to any stores!");
        println!("Go to 'Manage supply chain' to connect stores first.");
        wait_for_enter();
        return;
    }

    let product_id = match read_number("Enter product ID to transfer (0 to cancel): ") {
        Some(0) => return,
        Some(id) => id,
        None => {
            println!("Invalid product ID.");
            wait_for_enter();
            return;
        }
    };

    let factory = game.current_factory().unwrap();
    let available = factory.get_finished_good(product_id);
    if available == 0 {
        println!("No finished goods of that type.");
        wait_for_enter();
        return;
    }

    println!("Available: {}", available);
    let quantity = match read_number("Enter quantity to transfer: ") {
        Some(0) => return,
        Some(q) => q,
        None => {
            println!("Invalid quantity.");
            wait_for_enter();
            return;
        }
    };

    let store_num = match read_number("Enter store number to transfer to: ") {
        Some(0) => return,
        Some(n) if n > 0 && (n as usize) <= game.player.stores.len() => n as usize - 1,
        _ => {
            println!("Invalid store number.");
            wait_for_enter();
            return;
        }
    };

    match game.transfer_to_store(product_id, quantity, store_num) {
        Ok(actual) => {
            let product_name = game
                .get_product(product_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown");
            let store_name = &game.player.stores[store_num].name;
            println!();
            println!(
                "Transferred {} x {} to {}",
                actual, product_name, store_name
            );
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

/// Handles factory worker management
fn handle_factory_workers(game: &mut GameState) {
    if game.current_factory.is_none() {
        println!("No factory selected. Buy or select a factory first!");
        wait_for_enter();
        return;
    }

    loop {
        clear_screen();
        let factory = game.current_factory().unwrap();

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                  FACTORY WORKERS                             ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Factory: {:20}  │  Cash: ${:>10.2}   ║",
            factory.name, game.player.cash
        );
        println!("╠══════════════════════════════════════════════════════════════╣");

        if factory.workers.is_empty() {
            println!("║  No workers yet.                                             ║");
        } else {
            println!("║  Current Workers:                                            ║");
            for (idx, worker) in factory.workers.iter().enumerate() {
                println!(
                    "║    [{}] {:30} ${:.0}/day          ║",
                    idx + 1,
                    worker.name,
                    worker.salary
                );
            }
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Production slots: {} (base 2 + {} workers)                  ║",
            factory.production_slots(),
            factory.workers.len()
        );
        println!(
            "║  Total daily salaries: ${:>6.0}                               ║",
            factory.workers.iter().map(|w| w.salary).sum::<f64>()
        );
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [1] Hire worker ($75/day)                                   ║");
        println!("║  [2] Fire worker                                             ║");
        println!("║  [0] Back                                                    ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ");
        match input.trim() {
            "0" => return,
            "1" => {
                let factory = game.current_factory().unwrap();
                if factory.workers.len() >= 3 {
                    println!("Maximum of 3 workers per factory reached!");
                    wait_for_enter();
                    continue;
                }

                let name = read_input("Enter worker name (0 to cancel): ");
                if name == "0" || name.is_empty() {
                    continue;
                }

                match game.current_factory_mut().unwrap().hire_worker(&name) {
                    Ok(()) => {
                        println!();
                        println!("Hired: {}", name);
                        println!(
                            "New production slots: {}",
                            game.current_factory().unwrap().production_slots()
                        );
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
                wait_for_enter();
            }
            "2" => {
                let factory = game.current_factory().unwrap();
                if factory.workers.is_empty() {
                    println!("No workers to fire!");
                    wait_for_enter();
                    continue;
                }

                let worker_num =
                    match read_number("Enter worker number to fire (0 to cancel): ") {
                        Some(0) => continue,
                        Some(n) if n > 0 && (n as usize) <= factory.workers.len() => {
                            n as usize - 1
                        }
                        _ => {
                            println!("Invalid worker number.");
                            wait_for_enter();
                            continue;
                        }
                    };

                match game.current_factory_mut().unwrap().fire_worker(worker_num) {
                    Ok(fired) => {
                        println!();
                        println!("Fired: {}", fired.name);
                        println!(
                            "New production slots: {}",
                            game.current_factory().unwrap().production_slots()
                        );
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
                wait_for_enter();
            }
            _ => println!("Invalid choice."),
        }
    }
}

/// Handles switching between factories
fn handle_switch_factory(game: &mut GameState) {
    if game.player.factories.is_empty() {
        println!("You have no factories. Buy one first!");
        wait_for_enter();
        return;
    }

    if game.player.factories.len() == 1 {
        println!("You only have one factory.");
        wait_for_enter();
        return;
    }

    println!("Available factories:");
    for (idx, factory) in game.player.factories.iter().enumerate() {
        let current_marker = if Some(idx) == game.current_factory {
            " (current)"
        } else {
            ""
        };
        println!("  [{}] {}{}", idx + 1, factory.name, current_marker);
    }

    let factory_num = match read_number("Enter factory number (0 to cancel): ") {
        Some(0) => return,
        Some(n) if n > 0 && (n as usize) <= game.player.factories.len() => n as usize - 1,
        _ => {
            println!("Invalid factory number.");
            wait_for_enter();
            return;
        }
    };

    if game.switch_factory(factory_num).is_ok() {
        println!("Switched to: {}", game.player.factories[factory_num].name);
    }
    wait_for_enter();
}

/// Handles buying a new factory
fn handle_buy_new_factory(game: &mut GameState) {
    println!("Buy a new factory for $10,000");
    println!("Your cash: ${:.2}", game.player.cash);
    println!();

    if game.player.cash < 10000.0 {
        println!("Not enough cash! You need $10,000.");
        wait_for_enter();
        return;
    }

    let name = read_input("Enter name for new factory (0 to cancel): ");
    if name == "0" || name.is_empty() {
        return;
    }

    match game.buy_new_factory(&name) {
        Ok(()) => {
            println!();
            println!("SUCCESS! Purchased new factory: {}", name);
            println!("Remaining cash: ${:.2}", game.player.cash);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

// ==================== SUPPLY CHAIN MANAGEMENT ====================

/// Handles supply chain management for the current factory
fn handle_supply_chain(game: &mut GameState) {
    if game.current_factory.is_none() {
        println!("No factory selected. Buy or select a factory first!");
        wait_for_enter();
        return;
    }

    loop {
        clear_screen();
        let factory = game.current_factory().unwrap();

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    SUPPLY CHAIN                              ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Factory: {:40}       ║",
            factory.name
        );
        println!(
            "║  Auto-transfer: {:6}                                        ║",
            if factory.auto_transfer { "ON" } else { "OFF" }
        );
        println!("╠══════════════════════════════════════════════════════════════╣");

        // Show connected stores
        println!("║  Connected Stores:                                           ║");
        if factory.connected_stores.is_empty() {
            println!("║    (None - connect stores to enable transfers)               ║");
        } else {
            for store_id in &factory.connected_stores {
                if let Some(store_name) = game.get_store_name_by_id(*store_id) {
                    let is_primary = factory.primary_store() == Some(*store_id);
                    let marker = if is_primary { " [PRIMARY]" } else { "" };
                    println!("║    - {}{}                                     ║", store_name, marker);
                }
            }
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Available Stores:                                           ║");

        let factory = game.current_factory().unwrap();
        for (idx, store) in game.player.stores.iter().enumerate() {
            let connected = factory.is_connected_to(store.id);
            let status = if connected { "[CONNECTED]" } else { "" };
            println!(
                "║    [{}] {:30} {}           ║",
                idx + 1,
                store.name,
                status
            );
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [1] Connect store                                           ║");
        println!("║  [2] Disconnect store                                        ║");
        println!("║  [3] Toggle auto-transfer                                    ║");
        println!("║  [0] Back                                                    ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        if factory.auto_transfer && !factory.connected_stores.is_empty() {
            println!("Auto-transfer is ON: Finished goods will automatically ship");
            println!("to the primary connected store each day.");
            println!();
        }

        let input = read_input("Enter choice: ");
        match input.trim() {
            "0" => return,
            "1" => {
                // Connect store
                let store_num = match read_number("Enter store number to connect (0 to cancel): ") {
                    Some(0) => continue,
                    Some(n) if n > 0 && (n as usize) <= game.player.stores.len() => n as usize - 1,
                    _ => {
                        println!("Invalid store number.");
                        wait_for_enter();
                        continue;
                    }
                };

                let store_name = game.player.stores[store_num].name.clone();
                match game.connect_factory_to_store(store_num) {
                    Ok(()) => {
                        println!("Connected to {}!", store_name);
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
                wait_for_enter();
            }
            "2" => {
                // Disconnect store
                let factory = game.current_factory().unwrap();
                if factory.connected_stores.is_empty() {
                    println!("No stores connected.");
                    wait_for_enter();
                    continue;
                }

                let store_num = match read_number("Enter store number to disconnect (0 to cancel): ") {
                    Some(0) => continue,
                    Some(n) if n > 0 && (n as usize) <= game.player.stores.len() => n as usize - 1,
                    _ => {
                        println!("Invalid store number.");
                        wait_for_enter();
                        continue;
                    }
                };

                let store_name = game.player.stores[store_num].name.clone();
                match game.disconnect_factory_from_store(store_num) {
                    Ok(()) => {
                        println!("Disconnected from {}.", store_name);
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
                wait_for_enter();
            }
            "3" => {
                // Toggle auto-transfer
                let factory = game.current_factory().unwrap();
                if factory.connected_stores.is_empty() {
                    println!("Connect at least one store before enabling auto-transfer!");
                    wait_for_enter();
                    continue;
                }

                match game.toggle_factory_auto_transfer() {
                    Ok(enabled) => {
                        if enabled {
                            println!("Auto-transfer ENABLED!");
                            println!("Finished goods will automatically ship to the primary store.");
                        } else {
                            println!("Auto-transfer DISABLED.");
                        }
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
                wait_for_enter();
            }
            _ => println!("Invalid choice."),
        }
    }
}

// ==================== LOAN MANAGEMENT ====================

/// Handles loan management submenu
pub fn handle_manage_loans(game: &mut GameState) {
    loop {
        clear_screen();
        let economic_state = &game.market.economic_state;
        let base_rate = economic_state.interest_rate();

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                      MANAGE LOANS                            ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Your cash: ${:>10.2}    │    Total debt: ${:>10.2}    ║",
            game.player.cash,
            game.player.total_debt()
        );
        println!(
            "║  Max borrowable: ${:>10.2}  (Limit: ${:>10.2})          ║",
            game.player.max_borrowable(),
            Loan::MAX_TOTAL_DEBT
        );
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Economy: {:12}  │  Base interest rate: {:>5.1}%       ║",
            economic_state.name(),
            base_rate * 100.0
        );
        println!("╠══════════════════════════════════════════════════════════════╣");

        // Display current loans
        if game.player.loans.is_empty() {
            println!("║  No active loans.                                            ║");
        } else {
            println!("║  Active Loans:                                               ║");
            for loan in &game.player.loans {
                let loan_type_name = loan.loan_type.name();
                let days_info = match loan.days_remaining {
                    Some(days) => format!("{} days left", days),
                    None => "No term".to_string(),
                };
                println!(
                    "║    #{}: {} - ${:.2} @ {}  ({})    ║",
                    loan.id,
                    loan_type_name,
                    loan.balance,
                    loan.display_rate(),
                    days_info
                );
            }
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  [1] View all loans                                          ║");
        println!("║  [2] Take out a loan                                         ║");
        println!("║  [3] Make a payment                                          ║");
        println!("║  [4] View loan details                                       ║");
        println!("║  [0] Back to main menu                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let input = read_input("Enter choice: ");
        match input.trim() {
            "0" => return,
            "1" => display_all_loans(game),
            "2" => handle_take_loan(game),
            "3" => handle_make_payment(game),
            "4" => handle_view_loan_details(game),
            _ => println!("Invalid choice."),
        }
    }
}

/// Displays detailed info about all loans
fn display_all_loans(game: &GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                       ALL LOANS                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    if game.player.loans.is_empty() {
        println!("║  No active loans.                                            ║");
    } else {
        println!(
            "║  {:>3} {:15} {:>12} {:>10} {:>10}       ║",
            "ID", "Type", "Balance", "Rate", "Term"
        );
        println!("║  {:─>3} {:─>15} {:─>12} {:─>10} {:─>10}       ║", "", "", "", "", "");

        for loan in &game.player.loans {
            let term = match loan.days_remaining {
                Some(days) => format!("{} days", days),
                None => "-".to_string(),
            };
            println!(
                "║  {:>3} {:15} ${:>10.2} {:>9.1}% {:>10}       ║",
                loan.id,
                loan.loan_type.name(),
                loan.balance,
                loan.interest_rate * 100.0,
                term
            );
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Total debt: ${:>10.2}                                     ║",
            game.player.total_debt()
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    wait_for_enter();
}

/// Handles taking out a new loan
fn handle_take_loan(game: &mut GameState) {
    if game.player.max_borrowable() < Loan::MIN_LOAN {
        println!("You have reached your maximum debt limit!");
        wait_for_enter();
        return;
    }

    clear_screen();
    let economic_state = &game.market.economic_state;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     TAKE OUT A LOAN                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Economy: {:12}  │  Base rate: {:>5.1}%                  ║",
        economic_state.name(),
        economic_state.interest_rate() * 100.0
    );
    println!(
        "║  Max borrowable: ${:>10.2}                                 ║",
        game.player.max_borrowable()
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Loan Types:                                                 ║");
    println!("║                                                              ║");

    let flexible_rate = game.get_current_loan_rate(&LoanType::Flexible);
    let loc_rate = game.get_current_loan_rate(&LoanType::LineOfCredit);
    let term_rate = game.get_current_loan_rate(&LoanType::TermLoan);

    println!(
        "║  [1] Flexible Loan ({:.1}% annual)                            ║",
        flexible_rate * 100.0
    );
    println!("║      - Pay any amount anytime, no minimum payment            ║");
    println!("║      - Most flexibility, highest interest                    ║");
    println!("║                                                              ║");
    println!(
        "║  [2] Line of Credit ({:.1}% annual)                           ║",
        loc_rate * 100.0
    );
    println!("║      - Auto-deducts 2% of balance daily (min $10)            ║");
    println!("║      - Can pay extra manually, medium interest               ║");
    println!("║                                                              ║");
    println!(
        "║  [3] Term Loan ({:.1}% annual)                                ║",
        term_rate * 100.0
    );
    println!("║      - Full amount due at end of term (7/14/30 days)         ║");
    println!("║      - Lowest rate, penalty if you can't pay when due        ║");
    println!("║                                                              ║");
    println!("║  [0] Cancel                                                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let loan_type = loop {
        let input = read_input("Choose loan type (1-3, 0 to cancel): ");
        match input.trim() {
            "0" => return,
            "1" => break LoanType::Flexible,
            "2" => break LoanType::LineOfCredit,
            "3" => break LoanType::TermLoan,
            _ => println!("Invalid choice. Enter 1, 2, 3, or 0."),
        }
    };

    let max_loan = game.player.max_borrowable().min(Loan::MAX_LOAN);
    println!();
    println!(
        "Loan amount (${:.2} - ${:.2}):",
        Loan::MIN_LOAN,
        max_loan
    );

    let amount = match read_float("Enter amount: $") {
        Some(a) if a > 0.0 => a,
        _ => {
            println!("Invalid amount.");
            wait_for_enter();
            return;
        }
    };

    // For term loans, also ask for duration
    let days = if loan_type == LoanType::TermLoan {
        println!();
        println!("Term length:");
        println!("  [1] 7 days");
        println!("  [2] 14 days (-0.5% rate)");
        println!("  [3] 30 days (-1.0% rate)");

        let days = loop {
            let input = read_input("Choose term (1-3): ");
            match input.trim() {
                "1" => break 7u32,
                "2" => break 14u32,
                "3" => break 30u32,
                _ => println!("Invalid choice. Enter 1, 2, or 3."),
            }
        };
        Some(days)
    } else {
        None
    };

    // Confirm
    let rate = match loan_type {
        LoanType::Flexible => flexible_rate,
        LoanType::LineOfCredit => loc_rate,
        LoanType::TermLoan => {
            let base = term_rate;
            match days {
                Some(14) => (base - 0.005).max(0.01),
                Some(30) => (base - 0.01).max(0.01),
                _ => base,
            }
        }
    };

    println!();
    println!("Confirm loan:");
    println!("  Type: {}", loan_type.name());
    println!("  Amount: ${:.2}", amount);
    println!("  Annual Rate: {:.1}%", rate * 100.0);
    if let Some(d) = days {
        println!("  Term: {} days", d);
    }

    let confirm = read_input("Take this loan? [Y/n]: ");
    if confirm.to_lowercase() == "n" {
        return;
    }

    let result = match loan_type {
        LoanType::Flexible => game.take_flexible_loan(amount),
        LoanType::LineOfCredit => game.take_line_of_credit(amount),
        LoanType::TermLoan => game.take_term_loan(amount, days.unwrap()),
    };

    match result {
        Ok(loan_id) => {
            println!();
            println!("SUCCESS! Loan #{} approved.", loan_id);
            println!("${:.2} has been added to your cash.", amount);
            println!("New cash balance: ${:.2}", game.player.cash);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

/// Handles making a payment on a loan
fn handle_make_payment(game: &mut GameState) {
    if game.player.loans.is_empty() {
        println!("You have no active loans.");
        wait_for_enter();
        return;
    }

    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     MAKE A PAYMENT                           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Your cash: ${:>10.2}                                      ║",
        game.player.cash
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Your loans:                                                 ║");

    for loan in &game.player.loans {
        let loan_type_name = loan.loan_type.name();
        println!(
            "║    #{}: {} - Balance: ${:.2}                     ║",
            loan.id, loan_type_name, loan.balance
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let loan_id = match read_number("Enter loan # to pay (0 to cancel): ") {
        Some(0) => return,
        Some(id) => id,
        None => {
            println!("Invalid loan number.");
            wait_for_enter();
            return;
        }
    };

    let loan = match game.player.get_loan(loan_id) {
        Some(l) => l,
        None => {
            println!("Loan not found.");
            wait_for_enter();
            return;
        }
    };

    println!();
    println!("Loan #{} - {}", loan.id, loan.loan_type.name());
    println!("Current balance: ${:.2}", loan.balance);
    println!("Your cash: ${:.2}", game.player.cash);
    println!();
    println!("Enter payment amount (or 'all' to pay full balance):");

    let input = read_input("Amount: $");
    let amount = if input.to_lowercase() == "all" {
        loan.balance
    } else {
        match input.parse::<f64>() {
            Ok(a) if a > 0.0 => a,
            _ => {
                println!("Invalid amount.");
                wait_for_enter();
                return;
            }
        }
    };

    match game.make_loan_payment(loan_id, amount) {
        Ok(paid) => {
            println!();
            println!("Payment successful!");
            println!("Amount paid: ${:.2}", paid);

            // Check if loan was paid off
            if let Some(loan) = game.player.get_loan(loan_id) {
                println!("Remaining balance: ${:.2}", loan.balance);
            } else {
                println!("Loan has been paid off!");
            }
            println!("Your cash: ${:.2}", game.player.cash);

            // Clean up paid-off loans
            game.player.cleanup_loans();
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    wait_for_enter();
}

/// Displays detailed information about a specific loan
fn handle_view_loan_details(game: &GameState) {
    if game.player.loans.is_empty() {
        println!("You have no active loans.");
        wait_for_enter();
        return;
    }

    println!("Your loans:");
    for loan in &game.player.loans {
        println!("  #{}: {} - ${:.2}", loan.id, loan.loan_type.name(), loan.balance);
    }
    println!();

    let loan_id = match read_number("Enter loan # to view (0 to cancel): ") {
        Some(0) => return,
        Some(id) => id,
        None => {
            println!("Invalid loan number.");
            wait_for_enter();
            return;
        }
    };

    let loan = match game.player.get_loan(loan_id) {
        Some(l) => l,
        None => {
            println!("Loan not found.");
            wait_for_enter();
            return;
        }
    };

    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║                  LOAN #{} DETAILS                             ║",
        loan.id
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Type: {:40}           ║", loan.loan_type.name());
    println!("║  Description: {}  ║", loan.loan_type.description());
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Original Principal: ${:>10.2}                            ║", loan.principal);
    println!("║  Current Balance:    ${:>10.2}                            ║", loan.balance);
    println!(
        "║  Interest Accrued:   ${:>10.2}                            ║",
        loan.balance - loan.principal
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Annual Interest Rate: {:>6.2}%                              ║", loan.interest_rate * 100.0);
    println!(
        "║  Daily Interest Rate:  {:>6.4}%                             ║",
        loan.daily_rate() * 100.0
    );
    println!(
        "║  Daily Interest Cost:  ${:>8.2}                             ║",
        loan.balance * loan.daily_rate()
    );

    if loan.loan_type == LoanType::LineOfCredit {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Daily Auto-Payment:  ${:>10.2}                           ║",
            loan.get_auto_payment()
        );
    }

    if let Some(days) = loan.days_remaining {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Days Remaining: {:>5}                                      ║", days);
        if days == 0 {
            println!("║  STATUS: DUE NOW!                                            ║");
        } else if days <= 3 {
            println!("║  WARNING: Coming due soon!                                   ║");
        }
        println!(
            "║  Default Penalty (25%): ${:>10.2}                        ║",
            loan.default_penalty()
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    wait_for_enter();
}

/// Displays bankruptcy message
pub fn display_bankruptcy(game: &GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                       BANKRUPTCY!                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  Your business has run out of money!                         ║");
    println!("║  You can no longer pay your expenses.                        ║");
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Final Statistics:                                          ║");
    println!(
        "║    Days in business: {:>5}                                   ║",
        game.day - 1
    );
    println!(
        "║    Final cash: ${:>10.2}                                  ║",
        game.player.cash
    );
    println!(
        "║    Stores owned: {:>3}                                        ║",
        game.player.stores.len()
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Better luck next time!");
    println!();
}

/// Displays a goodbye message
pub fn display_goodbye(game: &GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    THANKS FOR PLAYING!                       ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Final Statistics:                                          ║");
    println!("║    Days played: {:>5}                                       ║", game.day - 1);
    println!("║    Final cash: ${:>10.2}                                  ║", game.player.cash);
    println!("║    Net worth: ${:>10.2}                                   ║", game.player.net_worth());
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

// ==================== INVESTMENT MANAGEMENT ====================

/// Handles the investment management submenu
pub fn handle_manage_investments(game: &mut GameState) {
    loop {
        clear_screen();
        display_investment_header(game);

        println!("Investment Options:");
        println!("  [1] View stock market");
        println!("  [2] View portfolio");
        println!("  [3] Buy stocks");
        println!("  [4] Sell stocks");
        println!("  [0] Back to main menu");
        println!();

        let input = read_input("Enter choice (0-4): ");
        match input.trim() {
            "1" => display_stock_market(game),
            "2" => display_portfolio(game),
            "3" => handle_buy_stocks(game),
            "4" => handle_sell_stocks(game),
            "0" => return,
            _ => {
                println!("Invalid choice.");
                wait_for_enter();
            }
        }
    }
}

fn display_investment_header(game: &GameState) {
    let portfolio_value = game.portfolio_value();
    let gain_loss = game.portfolio_gain_loss();
    let total_dividends = game.player.total_dividends_earned();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    INVESTMENT CENTER                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Cash: ${:>10.2}  │  Portfolio Value: ${:>10.2}       ║",
        game.player.cash, portfolio_value
    );

    let gain_label = if gain_loss >= 0.0 { "Gain" } else { "Loss" };
    println!(
        "║  Total {}: ${:>10.2}  │  Dividends Earned: ${:>8.2}  ║",
        gain_label, gain_loss.abs(), total_dividends
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

fn display_stock_market(game: &GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      STOCK MARKET                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:4} {:24} {:>8} {:>6} {:>8}   ║",
        "SYM", "Company", "Price", "Trend", "Type"
    );
    println!("║  {:─<4} {:─<24} {:─>8} {:─>6} {:─>8}   ║", "", "", "", "", "");

    for stock in &game.stock_market.stocks {
        let trend = stock.trend();
        let trend_str = format!("{:+.1}%", trend);
        println!(
            "║  {:4} {:24} ${:>7.2} {:>6} {:>8}   ║",
            stock.symbol,
            stock.name,
            stock.price,
            trend_str,
            stock.stock_type.name()
        );
    }

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Stock Types:                                                ║");
    println!("║    Blue Chip - Low risk, pays 4% annual dividends            ║");
    println!("║    Growth    - Medium risk, 1% dividends                     ║");
    println!("║    Speculative - High risk/reward, no dividends              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    wait_for_enter();
}

fn display_portfolio(game: &GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      YOUR PORTFOLIO                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    if game.player.portfolio.is_empty() {
        println!("║  You don't own any stocks yet.                               ║");
        println!("║  Use 'Buy stocks' to start investing!                        ║");
    } else {
        println!(
            "║  {:4} {:>6} {:>10} {:>10} {:>10} {:>8}  ║",
            "SYM", "Shares", "Avg Cost", "Cur Price", "Value", "Gain/Loss"
        );
        println!("║  {:─<4} {:─>6} {:─>10} {:─>10} {:─>10} {:─>8}  ║", "", "", "", "", "", "");

        let prices = game.get_stock_prices();
        let mut total_value = 0.0;
        let mut total_gain = 0.0;

        for (stock_id, holding) in &game.player.portfolio {
            if let Some(stock) = game.stock_market.get_stock(*stock_id) {
                let current_price = prices.get(stock_id).unwrap_or(&0.0);
                let value = holding.current_value(*current_price);
                let gain = holding.gain_loss(*current_price);
                total_value += value;
                total_gain += gain;

                let gain_str = if gain >= 0.0 {
                    format!("+${:.2}", gain)
                } else {
                    format!("-${:.2}", gain.abs())
                };

                println!(
                    "║  {:4} {:>6} ${:>9.2} ${:>9.2} ${:>9.2} {:>8}  ║",
                    stock.symbol,
                    holding.shares,
                    holding.avg_purchase_price,
                    current_price,
                    value,
                    gain_str
                );
            }
        }

        println!("║  {:─<4} {:─>6} {:─>10} {:─>10} {:─>10} {:─>8}  ║", "", "", "", "", "", "");

        let total_gain_str = if total_gain >= 0.0 {
            format!("+${:.2}", total_gain)
        } else {
            format!("-${:.2}", total_gain.abs())
        };

        println!(
            "║  {:4} {:>6} {:>10} {:>10} ${:>9.2} {:>8}  ║",
            "TOTL", "", "", "", total_value, total_gain_str
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    wait_for_enter();
}

fn handle_buy_stocks(game: &mut GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                       BUY STOCKS                             ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Your cash: ${:>10.2}                                     ║", game.player.cash);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:2} {:4} {:24} {:>10} {:>8}     ║",
        "ID", "SYM", "Company", "Price", "Type"
    );
    println!("║  {:─<2} {:─<4} {:─<24} {:─>10} {:─>8}     ║", "", "", "", "", "");

    for stock in &game.stock_market.stocks {
        println!(
            "║  {:>2} {:4} {:24} ${:>9.2} {:>8}     ║",
            stock.id,
            stock.symbol,
            stock.name,
            stock.price,
            stock.stock_type.name()
        );
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let stock_id = match read_number("Enter stock ID (0 to cancel): ") {
        Some(0) => return,
        Some(id) => id,
        None => {
            println!("Invalid ID.");
            wait_for_enter();
            return;
        }
    };

    let stock = match game.stock_market.get_stock(stock_id) {
        Some(s) => s,
        None => {
            println!("Stock not found.");
            wait_for_enter();
            return;
        }
    };

    let max_shares = (game.player.cash / stock.price) as u32;
    println!();
    println!(
        "Buying {} ({}) at ${:.2} per share",
        stock.name, stock.symbol, stock.price
    );
    println!("You can afford up to {} shares.", max_shares);
    println!();

    let shares = match read_number("Enter number of shares (0 to cancel): ") {
        Some(0) => return,
        Some(s) if s > 0 => s,
        _ => {
            println!("Invalid number.");
            wait_for_enter();
            return;
        }
    };

    match game.buy_stock(stock_id, shares) {
        Ok(total_cost) => {
            println!();
            println!(
                "SUCCESS! Bought {} shares for ${:.2}",
                shares, total_cost
            );
            println!("Remaining cash: ${:.2}", game.player.cash);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }

    wait_for_enter();
}

fn handle_sell_stocks(game: &mut GameState) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                       SELL STOCKS                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    if game.player.portfolio.is_empty() {
        println!("║  You don't own any stocks to sell.                           ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        wait_for_enter();
        return;
    }

    println!(
        "║  {:2} {:4} {:>8} {:>12} {:>12}             ║",
        "ID", "SYM", "Shares", "Cur Price", "Value"
    );
    println!("║  {:─<2} {:─<4} {:─>8} {:─>12} {:─>12}             ║", "", "", "", "", "");

    for (stock_id, holding) in &game.player.portfolio {
        if let Some(stock) = game.stock_market.get_stock(*stock_id) {
            let value = holding.current_value(stock.price);
            println!(
                "║  {:>2} {:4} {:>8} ${:>11.2} ${:>11.2}             ║",
                stock.id, stock.symbol, holding.shares, stock.price, value
            );
        }
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let stock_id = match read_number("Enter stock ID (0 to cancel): ") {
        Some(0) => return,
        Some(id) => id,
        None => {
            println!("Invalid ID.");
            wait_for_enter();
            return;
        }
    };

    let holding = match game.player.get_holding(stock_id) {
        Some(h) => h.clone(),
        None => {
            println!("You don't own this stock.");
            wait_for_enter();
            return;
        }
    };

    let stock = match game.stock_market.get_stock(stock_id) {
        Some(s) => s,
        None => {
            println!("Stock not found.");
            wait_for_enter();
            return;
        }
    };

    println!();
    println!(
        "Selling {} ({}) at ${:.2} per share",
        stock.name, stock.symbol, stock.price
    );
    println!("You own {} shares.", holding.shares);
    println!();

    let input = read_input("Enter number of shares to sell (or 'all'): ");
    let shares = if input.trim().to_lowercase() == "all" {
        holding.shares
    } else {
        match input.trim().parse::<u32>() {
            Ok(0) => return,
            Ok(s) => s,
            Err(_) => {
                println!("Invalid number.");
                wait_for_enter();
                return;
            }
        }
    };

    match game.sell_stock(stock_id, shares) {
        Ok(proceeds) => {
            let gain = proceeds - holding.avg_purchase_price * shares as f64;
            println!();
            println!("SUCCESS! Sold {} shares for ${:.2}", shares, proceeds);
            if gain >= 0.0 {
                println!("Profit: ${:.2}", gain);
            } else {
                println!("Loss: ${:.2}", gain.abs());
            }
            println!("New cash balance: ${:.2}", game.player.cash);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }

    wait_for_enter();
}
