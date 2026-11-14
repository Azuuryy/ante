use super::*;

fn row(centers: &[f32]) -> Vec<Home> {
    centers
        .iter()
        .enumerate()
        .map(|(index, &center)| Home {
            entity: Entity::from_raw_u32(index as u32 + 1).unwrap(),
            index,
            center,
            transform: UiTransform::IDENTITY,
        })
        .collect()
}

#[test]
fn passed_cards_shift_one_slot_toward_the_origin() {
    // Held card 1 moved to slot 3: cards 2 and 3 slide left, the others stay.
    let moved: Vec<usize> = (0..5).map(|index| shifted_index(index, 1, 3)).collect();
    assert_eq!(moved, [0, 1, 1, 2, 4]);
    // Held card 3 moved to slot 0: cards 0 to 2 slide right.
    let moved: Vec<usize> = (0..5).map(|index| shifted_index(index, 3, 0)).collect();
    assert_eq!(moved, [1, 2, 3, 3, 4]);
    assert_eq!(shifted_index(2, 2, 2), 2);
}

#[test]
fn the_held_card_takes_a_slot_once_it_passes_a_resting_centre() {
    let homes = row(&[100.0, 200.0, 300.0, 400.0]);
    // Card 1 nudged but not past a neighbour's centre stays in its slot.
    assert_eq!(held_target(&homes, 1, 290.0), 1);
    assert_eq!(held_target(&homes, 1, 110.0), 1);
    // Past card 2's centre it takes slot 2; past card 3's, the last slot.
    assert_eq!(held_target(&homes, 1, 310.0), 2);
    assert_eq!(held_target(&homes, 1, 450.0), 3);
    // Past card 0's centre to the left, the first slot.
    assert_eq!(held_target(&homes, 1, 90.0), 0);
}
