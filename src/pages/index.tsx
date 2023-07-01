import { useState } from "react";
import { CircuitContext, Proof } from "../components/Circuits";
import Image from "next/image";

export default function Home() {
  const [circuitIndex, setCircuitIndex] = useState(0);
  const [isValidProof, setIsValidProof] = useState("");
  const [input, setInput] = useState('{ "x": [5, 16, 8, 4, 2, 1]}');

  const clear = () => {
    setCircuitIndex(0);
    setIsValidProof("");
    localStorage.clear();
  };
  return (
    <main className="flex bg-gray-900 min-h-screen flex-col items-center justify-between p-24">
      <div className="columns-2 h-screen ">
        <CircuitContext
          circuitIndex={circuitIndex}
          setCircuitIndex={setCircuitIndex}
          clear={clear}
        />
        <div className="font-mono text-center h-full columns-1">
          <Proof
            isValidProof={isValidProof}
            setIsValidProof={setIsValidProof}
            input={input}
            setInput={setInput}
            circuitIndex={circuitIndex}
          />
        </div>
      </div>
    </main>
  );
}
