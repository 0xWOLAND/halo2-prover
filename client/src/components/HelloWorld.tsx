import { useContext } from "react";
import { WASMContext } from "@/context/wasm";

export const HelloWorld = async () => {
  const ctx = useContext(WASMContext);

  if (!ctx.wasm) {
    return <>...</>;
  }

  return (
    <div>
      <h1>Hello World oisdjoisjdfoiaj {ctx.wasm.hello_world()} Component</h1>
    </div>
  );
};
