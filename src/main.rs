use capitalism_tycoon::game::GameState;
use capitalism_tycoon::ui::{
    clear_screen, display_day_result, display_goodbye, display_header, display_menu,
    display_store, display_welcome, handle_buy_inventory, handle_set_prices, MenuChoice,
};

fn main() {
    // Initialize the game
    let mut game = GameState::new();

    // Show welcome screen
    display_welcome();

    // Main game loop
    loop {
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
                display_day_result(&result, game.day);
            }
            MenuChoice::Quit => {
                display_goodbye(&game);
                break;
            }
        }
    }
}
