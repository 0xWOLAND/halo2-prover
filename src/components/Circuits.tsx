import { useContext, useState, useEffect } from "react";
import { WASMContext } from "../context/wasm";
import Image from "next/image";
import ArithmeticCircuit from "../../public/arithmetic_circuit.svg";
import CollatzCircuit from "../../public/collatz.svg";

interface CircuitContextProps {
  setCircuitIndex: Function;
  circuitIndex: number;
  clear: Function;
}

interface ProofProps {
  isValidProof: String;
  setIsValidProof: Function;
  input: String;
  setInput: Function;
  circuitIndex: number;
}

export const CircuitContext = (props: CircuitContextProps) => {
  const ctx = useContext(WASMContext);
  const wasm = ctx.wasm!;
  let circuitIndex = props.circuitIndex;
  const setCircuitIndex = props.setCircuitIndex;
  const clear = props.clear;

  const images = [CollatzCircuit, ArithmeticCircuit];

  const handleSwitch = (e: number) => {
    clear();
    setCircuitIndex(
      (circuitIndex + e + wasm.get_circuit_count()) % wasm.get_circuit_count()
    );
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
          className="rounded-md bg-orange-300 m-2 py-1.5 px-3 text-slate-950"
          onClick={() => handleSwitch(-1)}
        >
          &lt;-
        </button>
        <button
          className="rounded-md bg-orange-300 m-2 py-1.5 px-3 text-slate-950"
          onClick={() => handleSwitch(1)}
        >
          -&gt;
        </button>
        <button
          className="rounded-md bg-red-600 m-2 py-1.5 px-3 text-slate-950"
          onClick={() => clear()}
        >
          Clear
        </button>
      </div>
    </div>
  );
};

export const Proof = (props: ProofProps) => {
  let isValidProof = props.isValidProof;
  let input = props.input;
  const setIsValidProof = props.setIsValidProof;
  const setInput = props.setInput;
  const circuitIndex = props.circuitIndex;

  const ctx = useContext(WASMContext);

  const wasm = ctx.wasm!;

  const getLocalItem = (s: string) => {
    return Uint8Array.from(
      (localStorage.getItem(s) as string).split(",").map((x) => parseInt(x))
    );
  };

  const setupParams = async () => {
    await localStorage.setItem("setup_params", wasm.setup(10).join(","));
  };

  const wasmGenerateProof = async () => {
    try {
      const setup_params = getLocalItem("setup_params");
      const witness = JSON.stringify(JSON.parse(input as string));
      localStorage.setItem(
        "proof",
        await wasm
          .wasm_generate_proof(setup_params, witness, circuitIndex)
          .join(",")
      );
    } catch {
      alert("Invalid params...");
    }
  };

  const wasmVerifyProof = async () => {
    try {
      const setup_params = getLocalItem("setup_params");
      const proof = getLocalItem("proof");
      const sequence = JSON.stringify(JSON.parse(input as string));
      const isValid = async () => {
        try {
          return wasm.wasm_verify_proof(
            setup_params,
            proof,
            sequence,
            circuitIndex
          );
        } catch {
          return false;
        }
      };
      setIsValidProof((await isValid()) ? "Valid Proof" : "Invalid Proof");
    } catch {
      alert("Invalid params...");
    }
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
      <div id="proofResult">{isValidProof}</div>
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
