use super::*;

#[test]
fn pack_sentinel_weight_equals_sum_of_real_entries() {
    let sum: f64 = PACKS[1..].iter().map(|(_, w)| w).sum();
    assert!(
        (sum - PACKS[0].1).abs() < 1e-9,
        "sentinel weight {} != sum of real entries {}",
        PACKS[0].1,
        sum
    );
}

#[test]
fn all_pool_names_resolve() {
    let mut failures: Vec<String> = Vec::new();
    for (label, unresolved) in [
        ("tarot", TAROTS_POOL.unresolved()),
        ("planet", PLANETS_POOL.unresolved()),
        ("spectral", SPECTRALS_POOL.unresolved()),
        ("common joker", COMMON_JOKERS_POOL.unresolved()),
        ("uncommon joker", UNCOMMON_JOKERS_POOL.unresolved()),
        ("rare joker", RARE_JOKERS_POOL.unresolved()),
        ("legendary joker", LEGENDARY_JOKERS_POOL.unresolved()),
        ("voucher", VOUCHERS_POOL.unresolved()),
        ("tag", TAGS_POOL.unresolved()),
        ("boss", BOSSES_POOL.unresolved()),
        ("enhancement", ENHANCEMENTS_POOL.unresolved()),
        ("card base", CARDS_POOL.unresolved()),
    ] {
        for name in unresolved {
            failures.push(format!("{label}: {name:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} pool name(s) failed to resolve against balatro_types:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
