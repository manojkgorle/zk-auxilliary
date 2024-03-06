// use core::marker::PhantomData;
use winterfell::{
    crypto::{hashers::Blake3_192, DefaultRandomCoin, ElementHasher},
    math::{fields::f128::BaseElement, FieldElement},
    FieldExtension::None,
    // FieldExtension::Quadratic,
    ProofOptions,
    Prover,
    Trace,
};

pub mod air;
pub mod prover;
pub mod trace_table;
pub mod verifier;
const TRACE_WIDTH: usize = 2;
use air::PermutationAir;
use prover::PermutationProver;
use trace_table::PermutationTraceTable;
// options / default options implementations are necessary for proof generation
fn main() {
    // configure logging
    if std::env::var("WINTER_LOG").is_err() {
        std::env::set_var("WINTER_LOG", "info");
    }
    let options = ProofOptions::new(28, 8, 16, None, 4, 31);
    let prover = PermutationProver::<Blake3_192<BaseElement>>::new(options.clone());
    let sequence = vec![4, 9, 10, 12, 456, 68, 1, 11];
    let permuted_sequence = vec![10, 12, 456, 68, 1, 11, 4, 9];
    let trace = prover.build_trace(sequence, permuted_sequence);
    let proof = prover.prove(trace).unwrap();
    // println!("{:?}", proof);
    let acceptable_options =
        winterfell::AcceptableOptions::OptionSet(vec![proof.options().clone()]);
    let result = winterfell::verify::<
        PermutationAir,
        Blake3_192<BaseElement>,
        DefaultRandomCoin<Blake3_192<BaseElement>>,
    >(proof, BaseElement::new(11), &acceptable_options)
    .unwrap_err();
    println!("{:?}", result);
}
