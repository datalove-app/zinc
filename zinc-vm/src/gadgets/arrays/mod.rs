use crate::gadgets;
use crate::gadgets::utils::math;
use crate::gadgets::Scalar;
use crate::Result;
use algebra::Field;
use r1cs_core::ConstraintSystem;

/// Select single value from array based on index bits.
///
/// **Note**: index bits are in **big-endian**.
pub fn recursive_select<F, CS>(
    mut cs: CS,
    index_bits_be: &[Scalar<F>],
    array: &[Scalar<F>],
) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    assert!(!array.is_empty(), "internal error in recursive_select 1");

    if array.len() == 1 {
        return Ok(array[0].clone());
    }

    assert!(
        !index_bits_be.is_empty(),
        "internal error in recursive_select 3"
    );

    // Skip unneeded upper bits, so we can always use the first bit for conditional select.
    let extra_bits = index_bits_be.len() - math::log2ceil(array.len());
    let index_bits_be = &index_bits_be[extra_bits..];

    let half = math::floor_to_power_of_two(array.len() - 1);
    let left = recursive_select(
        cs.ns(|| "left recursion"),
        &index_bits_be[1..],
        &array[..half],
    )?;
    let right = recursive_select(
        cs.ns(|| "right recursion"),
        &index_bits_be[1..],
        &array[half..],
    )?;

    gadgets::conditional_select(cs.ns(|| "select"), &index_bits_be[0], &right, &left)
}
