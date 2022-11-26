use crate::*;

pub fn random_number(recent_slothashes: &UncheckedAccount, seed: String) -> u64 {
    let data = recent_slothashes.data.borrow();
    let most_recent = array_ref![data, 12, 8];

    let clock = Clock::get().unwrap();
    // seed for the random number is a combination of the slot_hash - timestamp
    let seed = u64::from_le_bytes(*array_ref![seed.as_bytes(), 96, 8])
        .saturating_sub(u64::from_le_bytes(*most_recent).
            saturating_sub(clock.unix_timestamp as u64));

    return seed;
}