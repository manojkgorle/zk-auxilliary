use super::{ElementHasher, PermutationAir, PermutationTraceTable, Prover, Trace};
use core::marker::PhantomData;
use winterfell::crypto::DefaultRandomCoin;
use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;
use winterfell::{
    matrix::ColMatrix, AuxTraceRandElements, ConstraintCompositionCoefficients,
    DefaultConstraintEvaluator, DefaultTraceLde, ProofOptions, StarkDomain, TraceInfo,
    TracePolyTable,
};
pub struct PermutationProver<H: ElementHasher> {
    options: ProofOptions,
    _hasher: PhantomData<H>,
}

// implement trait Prover to this

impl<H: ElementHasher> PermutationProver<H> {
    pub fn new(options: ProofOptions) -> Self {
        Self {
            options,
            _hasher: PhantomData,
        }
    }

    pub fn build_trace(
        &self,
        sequence: Vec<u32>,
        permuted_sequence: Vec<u32>,
    ) -> PermutationTraceTable<BaseElement> {
        assert!(
            sequence.len().is_power_of_two(),
            "sequence length must be a power of 2",
        );
        assert!(
            sequence.len() == permuted_sequence.len(),
            "sequence and permuted sequence lengths should be equal"
        );
        let mut trace = PermutationTraceTable::new(2, sequence.len());
        trace.fill(
            |state| {
                state[0] = BaseElement::new(sequence[0] as u128);
                state[1] = BaseElement::new(permuted_sequence[0] as u128);
            },
            |index, state| {
                state[0] = BaseElement::new(sequence[index + 1] as u128);
                state[1] = BaseElement::new(permuted_sequence[index + 1] as u128);
            },
        );
        trace
    }
}

impl<H: ElementHasher> Prover for PermutationProver<H>
where
    H: ElementHasher<BaseField = BaseElement>,
{
    type Air = PermutationAir;
    type BaseField = BaseElement;
    type Trace = PermutationTraceTable<BaseElement>;
    type HashFn = H;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> = DefaultTraceLde<E, Self::HashFn>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> BaseElement {
        let last_step = trace.length() - 1;
        trace.get(0, last_step) //@todo check for pub inputs,here
    }
    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: AuxTraceRandElements<E>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }
}
