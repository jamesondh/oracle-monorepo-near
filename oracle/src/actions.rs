use near_bindgen::{BorshDeserialize, BorshSerialize};

#[BorshDeserialize, BorshSerialize]
enum Actions {
    DoThis,
    DoThat
}