use super::*;

// Smoke test: confirms the whole draw surface is stable and non-panicking.
#[test]
fn ante_one_smoke_test_is_stable() {
    fn run() -> (String, String, Vec<String>) {
        let mut inst = Instance::new("TESTSEED");
        inst.init_locks(1, false, true);
        inst.init_unlocks(1, false);

        let boss = format!("{:?}", inst.next_boss(1));
        let voucher = format!("{:?}", inst.next_voucher(1));

        let mut shop_items = Vec::new();
        for _ in 0..5 {
            let item = match inst.next_shop_item(1) {
                ShopItem::Joker(j) => format!("Joker({})", j.name()),
                ShopItem::Consumable(c) => format!("Consumable({})", c.name()),
                ShopItem::PlayingCard => "PlayingCard".to_string(),
            };
            shop_items.push(item);
        }

        let (category, size) = inst.next_pack(1);
        match category {
            PackCategory::Buffoon => {
                inst.next_buffoon_pack(pack_card_count(category, size), 1);
            }
            PackCategory::Arcana => {
                inst.next_arcana_pack(pack_card_count(category, size), 1);
            }
            _ => {}
        }

        (boss, voucher, shop_items)
    }

    let a = run();
    let b = run();
    assert_eq!(
        a, b,
        "same seed, same fixed draw sequence must repeat exactly"
    );
}
