use ante_core::{GameRng, evaluate, standard_deck};

fn main() {
    let mut rng = GameRng::new(0);
    let cards = standard_deck();
    println!(
        "Ante foundation: {} cards, first hand: {:?}, RNG sample: {}",
        cards.len(),
        evaluate(&cards[..5]).expect("five cards"),
        rng.next_u64()
    );
}
