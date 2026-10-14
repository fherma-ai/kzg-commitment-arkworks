//! A KZG commitment on BLS12-381, through arkworks' variable-base MSM.
//!
//! The reference string is a parameter of the point, so converting it is
//! `init`'s work and is not measured — which is the specification's whole
//! shape: in production the string is fixed and every client prepares it
//! once. This answer prepares it the plain way, as affine points, and leaves
//! the window choice to the library. A row that builds a table over the
//! string is a different answer to the same specification, and that is the
//! comparison the point exists for.
use ark_bls12_381::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
use ark_ff::{BigInt, PrimeField, Zero};

use crate::fherma::{Inputs, Outputs, Point, Tensor};

fn fq(data: &[u64], i: usize, limbs: usize) -> Fq {
    let mut words = [0u64; 6];
    words[..limbs].copy_from_slice(&data[i * limbs..(i + 1) * limbs]);
    Fq::from_bigint(BigInt::new(words)).expect("a base-field element below the modulus")
}

/// The `i`-th coefficient. The signature gives every value the base field's
/// six-word stride and the scalar field is four words wide, so the words
/// above it must be zero.
fn fr(data: &[u64], i: usize, limbs: usize) -> Fr {
    let row = &data[i * limbs..(i + 1) * limbs];
    assert!(
        row[4.min(limbs)..].iter().all(|word| *word == 0),
        "a coefficient wider than the scalar field",
    );
    let mut words = [0u64; 4];
    let take = limbs.min(4);
    words[..take].copy_from_slice(&row[..take]);
    Fr::from_bigint(BigInt::new(words)).expect("a coefficient below the group order")
}

fn row(value: Fq, limbs: usize) -> Tensor<u64> {
    Tensor { shape: vec![limbs as i64], data: value.into_bigint().0[..limbs].to_vec() }
}

pub struct State {
    limbs: usize,
    /// The reference string as the library wants it. Prepared once, outside
    /// the measurement, because the point says the string is fixed.
    bases: Vec<G1Affine>,
}

impl Default for State {
    fn default() -> Self {
        State { limbs: 6, bases: Vec::new() }
    }
}

pub fn init(p: &Point) -> State {
    let limbs = p.L as usize;
    let n = p.N as usize;
    // `new_unchecked`: the bundle takes every point of the string from a
    // square root of the curve equation and clears the cofactor, so each one
    // is in G1 by construction.
    let bases = (0..n)
        .map(|i| G1Affine::new_unchecked(fq(&p.srs_x.data, i, limbs), fq(&p.srs_y.data, i, limbs)))
        .collect();
    State { limbs, bases }
}

pub fn run(state: &mut State, inp: &Inputs) -> Outputs {
    let limbs = state.limbs;
    let n = state.bases.len();
    let scalars: Vec<Fr> = (0..n).map(|i| fr(&inp.coeffs.data, i, limbs)).collect();

    let commitment = G1Projective::msm(&state.bases, &scalars)
        .expect("one coefficient for every point of the string")
        .into_affine();

    let (x, y) = commitment.xy().unwrap_or((Fq::zero(), Fq::zero()));
    Outputs { cx: row(x, limbs), cy: row(y, limbs) }
}

pub fn free(state: State) {
    drop(state);
}
