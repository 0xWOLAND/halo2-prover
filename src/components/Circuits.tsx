import { useContext } from "react";
import { WASMContext } from "../context/wasm";

export const Halo2Circuits = () => {
  return (
    <div className="h-full ">
      <img src="collatz.svg" className="h-full " />
    </div>
  );
};

// pub fn setup(k: u32) -> Uint8Array
// pub fn wasm_generate_proof(_params: &[u8], _sequence: &[u8]) -> Uint8Array
// pub fn wasm_verify_proof(_params: &[u8], proof: &[u8]) -> bool

export const Proof = () => {
  const ctx = useContext(WASMContext);

  if (!ctx.wasm) {
    console.log("this is not working!");
    return <>ahahhh</>;
  }

  const setup_params = ctx.wasm.setup(10);
  const sequence = Uint8Array.from([5, 16, 8, 4, 2, 1]);
  console.log("Sequence");
  console.log(sequence);
  const proof = ctx.wasm.wasm_generate_proof(setup_params, sequence);
  console.log("Proof");
  console.log(proof);
  const isValid = ctx.wasm.wasm_verify_proof(setup_params, proof);
  console.log("isValid");
  console.log(isValid);

  return (
    <div className="columns-1">
      <h1>{ctx.wasm.hello_world()}</h1>
      <div className="container mx-auto">
        <button className="rounded-md bg-orange-300 m-2 py-1.5 px-2 w-56 text-slate-950">
          Setup Params
        </button>
      </div>
      <div className="container mx-auto">
        <button className="rounded-md bg-orange-300 m-2 py-1.5 px-2 w-56 text-slate-950">
          Generate Proof
        </button>
      </div>
      <div className="container mx-auto">
        <button className="rounded-md bg-orange-300 m-2 py-1.5 px-2 w-56 text-slate-950">
          Verify Proof
        </button>
      </div>
    </div>
  );
};
