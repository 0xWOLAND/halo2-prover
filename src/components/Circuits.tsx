import { useContext, useState, useEffect } from "react";
import { WASMContext } from "../context/wasm";
import Image from "next/image";
import ArithmeticCircuit from "../../public/arithmetic_circuit.svg";
import CollatzCircuit from "../../public/collatz.svg";

// set current circuit in local storage
// every time you switch it clears local storage
export const Halo2Circuits = () => {
  const ctx = useContext(WASMContext);
  const wasm = ctx.wasm!;
  const [circuitIndex, setCircuitIndex] = useState(0);

  const images = [CollatzCircuit, ArithmeticCircuit];

  useEffect(() => {
    localStorage.setItem("circuit_index", circuitIndex.toString());
  });

  const handleSwitch = (e: number) => {
    setCircuitIndex(
      (circuitIndex + e + wasm.get_circuit_count()) % wasm.get_circuit_count()
    );
    localStorage.setItem("circuit_index", circuitIndex.toString());
  };

  return (
    <div className="h-full columns-1 items-start ">
      <Image
        className="shadow-slate-600 shadow-sm m-1"
        src={images[circuitIndex]}
        alt="Collatz Arithmetic Circuit"
      />
      <div className="flex justify-center">
        <button
          className="rounded-md bg-orange-300 m-2 py-1.5 px-2 text-slate-950"
          onClick={() => handleSwitch(-1)}
        >
          &lt;-
        </button>
        <button
          className="rounded-md bg-orange-300 m-2 py-1.5 px-2 text-slate-950"
          onClick={() => handleSwitch(1)}
        >
          -&gt;
        </button>
      </div>
    </div>
  );
};

export const Proof = () => {
  const [isValidProof, setIsValidProof] = useState(false);
  const [input, setInput] = useState('{ "x": [5, 16, 8, 4, 2, 1]}');

  const ctx = useContext(WASMContext);

  const wasm = ctx.wasm!;

  const getLocalItem = (s: string) => {
    return Uint8Array.from(
      (localStorage.getItem(s) as string).split(",").map((x) => parseInt(x))
    );
  };

  const setupParams = () => {
    localStorage.setItem("setup_params", wasm.setup(10).join(","));
    console.log(wasm.hello_world());
  };

  const wasmGenerateProof = () => {
    const setup_params = getLocalItem("setup_params");
    const sequence = JSON.stringify(JSON.parse(input));
    const circuit_index = parseInt(
      localStorage.getItem("circuit_index") as string
    );
    localStorage.setItem(
      "proof",
      wasm.wasm_generate_proof(setup_params, sequence, circuit_index).join(",")
    );
  };

  const wasmVerifyProof = () => {
    console.log("1");
    const setup_params = getLocalItem("setup_params");
    console.log("2");
    const proof = getLocalItem("proof");
    console.log("3");
    const circuitIndex = parseInt(
      localStorage.getItem("circuit_index") as string
    );
    console.log("4");
    const sequence = JSON.stringify(JSON.parse(input));
    console.log("5");
    const isValid: boolean = wasm.wasm_verify_proof(
      setup_params,
      proof,
      sequence,
      circuitIndex
    );
    setIsValidProof(isValid);
  };

  return (
    <div className="columns-1">
      <div className="mb-6 ">
        <textarea
          id="input_field"
          className="max-h-96 h-60 block w-full p-2.5 text-sm text-gray-50 bg-gray-700 rounded-lg border border-gray-300"
          onChange={(e) => setInput(e.target.value)}
          placeholder='{ "x": [5, 16, 8, 4, 2, 1, 1]}'
        ></textarea>
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
