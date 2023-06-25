// use halo2_proofs::{arithmetic::Field, halo2curves::FieldExt};
// use num_bigint::BigUint;
//
// fn from_bytes_le(bytes: &[u8]) -> FieldExt {
// let mut repr = Self::Repr::default();
// repr.as_mut()[..bytes.len()].copy_from_slice(bytes);
// FieldExt::from_repr(repr).unwrap()
// }
//
// pub fn biguint_to_fe<F: FieldExt>(e: &BigUint) -> F {
// let bytes = e.to_bytes_le();
// F::from_bytes_le(&bytes)
// }
//
// pub fn fe_to_biguint<F: FieldExt>(fe: &F) -> BigUint {
// BigUint::from_bytes_le(fe.to_bytes_le().as_ref())
// }
//
