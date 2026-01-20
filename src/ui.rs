use crate::economy::Market;
use crate::game::{DayResult, GameState};
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
    println!("  [7] Quit game");
    println!();

    loop {
        let input = read_input("Enter choice (1-7): ");
        match input.trim() {
            "1" => return MenuChoice::ViewStore,
            "2" => return MenuChoice::BuyInventory,
            "3" => return MenuChoice::SetPrices,
            "4" => return MenuChoice::AdvanceDay,
            "5" => return MenuChoice::ManageStores,
            "6" => return MenuChoice::ManageStaff,
            "7" => return MenuChoice::Quit,
            _ => println!("Invalid choice. Please enter 1-7."),
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

/// Displays available products for purchase
pub fn display_buy_menu(game: &GameState) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  WHOLESALE MARKET                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  {:3} {:20} {:>12} {:>15}        ║", "ID", "Product", "Price", "Category");
    println!("║  {:─<3} {:─<20} {:─>12} {:─>15}        ║", "", "", "", "");

    for product in &game.products {
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
pub fn display_day_result(result: &DayResult, new_day: u32) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║                     DAY {} RESULTS                            ║",
        new_day - 1
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Sales section
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

    // Expenses section
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  EXPENSES:                                                   ║");
    for (store_name, rent, salaries) in &result.expenses_by_store {
        println!(
            "║    {}: Rent ${:.0} + Salaries ${:.0}                   ║",
            store_name, rent, salaries
        );
    }
    println!(
        "║    Total Expenses: ${:>10.2}                               ║",
        result.total_expenses
    );

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
        Ok(()) => {
            println!();
            println!("SUCCESS! Purchased new store: {}", name);
            println!("Remaining cash: ${:.2}", game.player.cash);
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
