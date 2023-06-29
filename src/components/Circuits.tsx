import { useContext, useState } from "react";
import { WASMContext } from "../context/wasm";

export const Halo2Circuits = () => {
  return (
    <div className="h-full ">
      <img src="collatz.svg" className="h-full " />
    </div>
  );
};
export const Proof = () => {
  const [isValidProof, setIsValidProof] = useState(false);
  const ctx = useContext(WASMContext);

  const getLocalItem = (s: string) => {
    return Uint8Array.from(
      (localStorage.getItem(s) as string).split(",").map((x) => parseInt(x))
    );
  };

  const setupParams = () => {
    localStorage.setItem("setup_params", ctx.wasm.setup(10));
  };

  const wasmGenerateProof = () => {
    const setup_params = getLocalItem("setup_params");
    const sequence = Uint8Array.from([5, 16, 8, 4, 2, 1]);
    localStorage.setItem(
      "proof",
      ctx.wasm.wasm_generate_proof(setup_params, sequence)
    );
  };

  const wasmVerifyProof = () => {
    const setup_params = getLocalItem("setup_params");
    const proof = getLocalItem("proof");
    const isValid: boolean = ctx.wasm.wasm_verify_proof(setup_params, proof);
    setIsValidProof(isValid);
  };

  return (
    <div className="columns-1">
      <div id="proofResult">{isValidProof ? "yes" : "no"}</div>
      <div className="container mx-auto">
        <button
          className="rounded-md bg-orange-300 m-2 py-1.5 px-2 w-56 text-slate-950"
          onClick={setupParams}
        >
          Setup Params
        </button>
      </div>
      <div className="container mx-auto">
        <button
          className="rounded-md bg-orange-300 m-2 py-1.5 px-2 w-56 text-slate-950"
          onClick={wasmGenerateProof}
        >
          Generate Proof
        </button>
      </div>
      <div className="container mx-auto">
        <button
          className="rounded-md bg-orange-300 m-2 py-1.5 px-2 w-56 text-slate-950"
          onClick={wasmVerifyProof}
        >
          Verify Proof
        </button>
      </div>
    </div>
  );
};
