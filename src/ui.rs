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
    Quit,
}

/// Clears the screen (simple version)
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Displays the game header with status
pub fn display_header(game: &GameState) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              BUSINESS TYCOON - Rust Edition                  ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Day: {:>4}  │  Cash: ${:>10.2}  │  Net Worth: ${:>10.2}  ║",
        game.day,
        game.player.cash,
        game.player.net_worth()
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
    println!("  [5] Quit game");
    println!();

    loop {
        let input = read_input("Enter choice (1-5): ");
        match input.trim() {
            "1" => return MenuChoice::ViewStore,
            "2" => return MenuChoice::BuyInventory,
            "3" => return MenuChoice::SetPrices,
            "4" => return MenuChoice::AdvanceDay,
            "5" => return MenuChoice::Quit,
            _ => println!("Invalid choice. Please enter 1-5."),
        }
    }
}

/// Displays the store inventory
pub fn display_store(game: &GameState) {
    let store = game.player.store();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    {} - Inventory                    ║", store.name);
    println!("╠══════════════════════════════════════════════════════════════╣");

    if store.inventory.is_empty() {
        println!("║  (No inventory yet - buy some products!)                     ║");
    } else {
        println!("║  {:20} {:>8} {:>12} {:>12}      ║", "Product", "Qty", "Retail $", "Markup %");
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

/// Handles buying inventory - loops until user chooses to exit
pub fn handle_buy_inventory(game: &mut GameState) {
    loop {
        clear_screen();
        display_buy_menu(game);

        println!("Your cash: ${:.2}", game.player.cash);
        println!();

        let product_id = match read_number("Enter product ID (0 to return to menu): ") {
            Some(0) => return,
            Some(id) => id,
            None => {
                println!("Invalid product ID.");
                continue;
            }
        };

        if game.get_product(product_id).is_none() {
            println!("Product not found.");
            continue;
        }

        let quantity = match read_number("Enter quantity (0 to cancel): ") {
            Some(0) => continue,
            Some(q) => q,
            None => {
                println!("Invalid quantity.");
                continue;
            }
        };

        match game.buy_inventory(product_id, quantity) {
            Ok(cost) => {
                let product_name = game.get_product(product_id).map(|p| p.name.clone()).unwrap_or_default();
                println!();
                println!("SUCCESS! Bought {} x {} for ${:.2}", quantity, product_name, cost);
                println!("Remaining cash: ${:.2}", game.player.cash);
            }
            Err(e) => {
                println!();
                println!("ERROR: {}", e);
            }
        }

        println!();
        let choice = read_input("Buy another? [Y/n]: ");
        if choice.to_lowercase() == "n" {
            return;
        }
    }
}

/// Handles setting retail prices - loops until user chooses to exit
pub fn handle_set_prices(game: &mut GameState) {
    loop {
        clear_screen();
        let store = game.player.store();

        if store.inventory.is_empty() {
            println!("You have no inventory to price. Buy some products first!");
            wait_for_enter();
            return;
        }

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    SET RETAIL PRICES                         ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  {:3} {:20} {:>10} {:>10} {:>10}   ║", "ID", "Product", "Wholesale", "Current", "Markup");
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
    println!("║                     DAY {} RESULTS                            ║", new_day - 1);
    println!("╠══════════════════════════════════════════════════════════════╣");

    if result.sales_by_product.is_empty() {
        println!("║  No sales today. Check your prices or stock!                 ║");
    } else {
        for (name, qty, revenue) in &result.sales_by_product {
            println!("║  Sold {:>3} x {:20} = ${:>10.2}          ║", qty, name, revenue);
        }
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║  TOTAL: {} items sold for ${:.2}                       ║",
            result.total_items_sold, result.total_revenue
        );
    }

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
