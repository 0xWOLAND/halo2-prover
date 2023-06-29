import { useContext, useState } from "react";
import { WASMContext } from "../context/wasm";
import { InputField } from "./Input";

export const Halo2Circuits = () => {
  return (
    <div className="h-full flex items-start">
      <img src="collatz.svg" />
    </div>
  );
};
export const Proof = () => {
  const [isValidProof, setIsValidProof] = useState(false);
  const [input, setInput] = useState("");

  const ctx = useContext(WASMContext);

  if (!ctx.wasm) {
    return <></>;
  }

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
  const json_entry =
    '{ "name": "John Doe", "age": 43, "phones": [ "+44 1234567", "+44 2345678" ] }';

  return (
    <div className="columns-1">
      <div className="mb-6 ">
        <textarea
          id="input_field"
          className="max-h-96 h-60 block w-full p-2.5 text-sm text-gray-50 bg-gray-700 rounded-lg border border-gray-300"
          onChange={(e) => setInput(e.target.value)}
        >
          {json_entry}
        </textarea>
      </div>
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
