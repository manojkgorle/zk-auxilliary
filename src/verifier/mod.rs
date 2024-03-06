// use winterfell::{verify, AcceptableOptions::OptionSet, StarkProof, VerifierError};

// use super::{DefaultRandomCoin, PermutationAir};

// fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
//     let acceptable_options =
//         winterfell::AcceptableOptions::OptionSet(vec![proof.options().clone()]);

//     winterfell::verify::<PermuationAir, H, DefaultRandomCoin<H>>(
//         proof,
//         self.result,
//         &acceptable_options,
//     )
// }
