import { Halo2Circuits, Proof } from "../components/Circuits";
import Image from "next/image";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="columns-2 h-screen ">
        <Halo2Circuits />
        <div className="font-mono text-center h-full columns-1">
          <Proof />
        </div>
      </div>
    </main>
  );
}
