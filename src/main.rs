mod game_view_2048;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<game_view_2048::GameState>();
}