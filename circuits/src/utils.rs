use halo2_proofs::{
    dev::MockProver,
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        pasta::EqAffine,
    },
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Circuit, Error, ProvingKey, VerifyingKey,
    },
    poly::{
        commitment::{Params, ParamsProver},
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverGWC, ProverSHPLONK, VerifierGWC, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use plotters::prelude::*;
use rand_core::OsRng;
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(target_family = "wasm"))]
pub fn draw_graph(k: u32, name: String, circuit: &impl Circuit<Fr>) {
    use halo2_proofs::dev::CircuitLayout;

    let root = SVGBackend::new(&name, (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled(&name, ("sans-serif", 30)).unwrap();

    CircuitLayout::default()
        .show_equality_constraints(true)
        .view_width(0..2)
        .view_height(0..16)
        .show_labels(true)
        .render(k, circuit, &root)
        .unwrap()
}

pub fn run_mock_prover(
    k: u32,
    circuit: &impl Circuit<Fr>,
    public_input: &Vec<Fr>,
) -> Result<(), Vec<halo2_proofs::dev::VerifyFailure>> {
    let pub_inp = {
        if public_input.len() > 0 {
            vec![public_input.clone()]
        } else {
            vec![]
        }
    };
    let prover = MockProver::run(k, circuit, pub_inp).expect("Mock prover should run");

    prover.verify()
}

pub fn generate_params(k: u32) -> ParamsKZG<Bn256> {
    ParamsKZG::<Bn256>::new(k)
}

pub fn generate_keys(
    params: &ParamsKZG<Bn256>,
    circuit: &impl Circuit<Fr>,
) -> (ProvingKey<G1Affine>, VerifyingKey<G1Affine>) {
    let vk = keygen_vk(params, circuit).expect("vk should not fail");
    let pk = keygen_pk(params, vk.clone(), circuit).expect("keygen_pk should not fail");
    (pk, vk)
}

pub fn generate_proof(
    params: &ParamsKZG<Bn256>,
    pk: &ProvingKey<G1Affine>,
    circuit: impl Circuit<Fr>,
) -> Vec<u8> {
    println!("Generating proof...");

    let mut transcript: Blake2bWrite<Vec<u8>, _, Challenge255<_>> =
        Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

    let instances: &[&[Fr]] = &[&[]];
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<_>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, _>,
        _,
    >(params, pk, &[circuit], &[&[]], OsRng, &mut transcript)
    .expect("Prover should not fail");
    transcript.finalize()
}

pub fn generate_proof_with_instance(
    params: &ParamsKZG<Bn256>,
    pk: &ProvingKey<G1Affine>,
    circuit: impl Circuit<Fr>,
    public_input: &[Fr],
) -> Vec<u8> {
    println!("Generating proof...");
    println!("Public input: {:?}", public_input);
    let mut transcript: Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>> =
        Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverGWC<'_, Bn256>,
        Challenge255<G1Affine>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<_>>,
        _,
    >(
        &params,
        &pk,
        &[circuit],
        &[&[&public_input]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation should not fail");
    transcript.finalize()
    // Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..])
}

pub fn verify(
    params: &ParamsKZG<Bn256>,
    pk: &ProvingKey<G1Affine>,
    proof: &Vec<u8>,
) -> Result<(), Error> {
    println!("Verifying proof...");
    let strategy = SingleStrategy::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(params, pk.get_vk(), strategy, &[&[]], &mut transcript)
}
pub fn verify_with_instance(
    params: &ParamsKZG<Bn256>,
    pk: &ProvingKey<G1Affine>,
    proof: &Vec<u8>,
    public_input: &[Fr],
) -> Result<(), Error> {
    println!("Verifying proof...");
    let transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    let strategy = SingleStrategy::new(&params);
    Ok(verify_proof::<_, VerifierGWC<Bn256>, _, _, _>(
        &params,
        pk.get_vk(),
        strategy.clone(),
        &[&[&public_input]],
        &mut transcript.clone(),
    )
    .unwrap())
}
