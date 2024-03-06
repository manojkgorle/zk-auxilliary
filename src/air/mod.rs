use std::vec;

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use winterfell::{
    Air, AirContext, Assertion, EvaluationFrame, TraceInfo, TransitionConstraintDegree,
};

// VM AIR
// =============================================================================================

pub struct PermutationAir {
    context: AirContext<BaseElement>,
    result: BaseElement, // this is the true result of the computation, user defined
}

impl Air for PermutationAir {
    type BaseField = BaseElement;
    type PublicInputs = BaseElement;
    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        //@todo sometimes we may not have any public inputs. How to deal with that case?
        assert_eq!(TRACE_WIDTH + 1, trace_info.width());
        PermutationAir {
            context: AirContext::new_multi_segment(
                trace_info,
                vec![TransitionConstraintDegree::new(1)],
                vec![TransitionConstraintDegree::new(3)],
                1,
                1,
                options,
            ),
            result: pub_inputs,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());
        //total contraints are zero
        result[0] = BaseElement::ZERO.into();
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // no assertions
        let last_step = self.trace_length() - 1;
        vec![Assertion::single(0, last_step, self.result)]
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        _periodic_values: &[F],
        aux_rand_elements: &winterfell::AuxTraceRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + winterfell::math::ExtensionOf<F>,
    {
        // let main_current = main_frame.current();
        let main_next = main_frame.next();
        let aux_current = aux_frame.current();
        let aux_next = aux_frame.next();
        let random_elements = aux_rand_elements.get_segment_elements(0);

        // row of a result for this pair of trace_frames
        result[0] = aux_next[0]
            - aux_current[0]
                * (<F as Into<E>>::into(main_next[0]) + random_elements[0])
                * ((<F as Into<E>>::into(main_next[1]) + random_elements[0]).inv());
        // println!("hey: {:?}", result[0]);
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        _aux_rand_elements: &winterfell::AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let last_step = self.trace_length() - 1;
        vec![Assertion::single(0, last_step, E::ONE)]
    }
}
