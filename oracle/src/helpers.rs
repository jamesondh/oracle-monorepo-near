use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

/*** operators that does not take decimals into account ***/
pub fn calc_product(a: u128, b: u128, divisor: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let divisor_u256 = u256::from(divisor);

    (a_u256 * b_u256 / divisor_u256).as_u128()
}