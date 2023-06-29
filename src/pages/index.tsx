import { Halo2Circuits, Proof } from "../components/Circuits";
import Image from "next/image";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="columns-2">
        <Halo2Circuits />
        <div className="font-mono text-center columns-1">
          <h1>halo2-prover</h1>
          <Proof />
        </div>
      </div>
    </main>
  );
}
