//! A KZG commitment: the polynomial's coefficients against the reference
//! string, which is one multi-scalar multiplication and nothing else.
use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
use ark_ff::{BigInt, PrimeField, Zero};

use crate::fherma::{Inputs, Outputs, Point, Tensor};

/// The `i`-th row of a limb tensor as an element of the field `F`.
fn fe<F: PrimeField<BigInt = BigInt<4>>>(data: &[u64], i: usize, limbs: usize) -> F {
    let mut words = [0u64; 4];
    words[..limbs].copy_from_slice(&data[i * limbs..(i + 1) * limbs]);
    F::from_bigint(BigInt::new(words)).expect("a field element below the modulus")
}

fn row<F: PrimeField<BigInt = BigInt<4>>>(value: F, limbs: usize) -> Tensor<u64> {
    Tensor { shape: vec![limbs as i64], data: value.into_bigint().0[..limbs].to_vec() }
}

#[derive(Default)]
pub struct State {
    limbs: usize,
}

pub fn init(p: &Point) -> State {
    State { limbs: p.L as usize }
}

pub fn run(state: &mut State, inp: &Inputs) -> Outputs {
    let limbs = state.limbs;
    let n = inp.coeffs.data.len() / limbs;

    let scalars: Vec<Fr> = (0..n).map(|i| fe(&inp.coeffs.data, i, limbs)).collect();
    // `new_unchecked`: the generator takes every point of the string from a
    // square root of the curve equation, and BN254 has cofactor one.
    let bases: Vec<G1Affine> = (0..n)
        .map(|i| {
            G1Affine::new_unchecked(
                fe::<Fq>(&inp.srs_x.data, i, limbs),
                fe::<Fq>(&inp.srs_y.data, i, limbs),
            )
        })
        .collect();

    let c = G1Projective::msm(&bases, &scalars)
        .expect("one coefficient for every point of the string")
        .into_affine();
    let (x, y) = c.xy().unwrap_or((Fq::zero(), Fq::zero()));
    Outputs { cx: row(x, limbs), cy: row(y, limbs) }
}

pub fn free(state: State) {
    drop(state);
}
