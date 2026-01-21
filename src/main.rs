use capitalism_tycoon::game::GameState;
use capitalism_tycoon::ui::{
    clear_screen, display_bankruptcy, display_day_result, display_goodbye, display_header,
    display_menu, display_store, display_welcome, handle_buy_inventory, handle_manage_factories,
    handle_manage_staff, handle_manage_stores, handle_set_prices, MenuChoice,
};

fn main() {
    // Initialize the game
    let mut game = GameState::new();

    // Show welcome screen
    display_welcome();

    // Main game loop
    loop {
        // Check for bankruptcy
        if game.is_bankrupt {
            display_bankruptcy(&game);
            break;
        }

        clear_screen();
        display_header(&game);

        match display_menu() {
            MenuChoice::ViewStore => {
                display_store(&game);
            }
            MenuChoice::BuyInventory => {
                handle_buy_inventory(&mut game);
            }
            MenuChoice::SetPrices => {
                handle_set_prices(&mut game);
            }
            MenuChoice::AdvanceDay => {
                let result = game.advance_day();
                display_day_result(&result, game.day, &game);
            }
            MenuChoice::ManageStores => {
                handle_manage_stores(&mut game);
            }
            MenuChoice::ManageStaff => {
                handle_manage_staff(&mut game);
            }
            MenuChoice::ManageFactories => {
                handle_manage_factories(&mut game);
            }
            MenuChoice::Quit => {
                display_goodbye(&game);
                break;
            }
        }
    }
}
