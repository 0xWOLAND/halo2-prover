import { useContext } from "react";
import { WASMContext } from "../context/wasm";

export const HelloWorld = () => {
  const ctx = useContext(WASMContext);

  if (!ctx.wasm) {
    console.log("this is not working!");
    return <>ahahhh</>;
  }

  return <>{ctx.wasm.hello_world()}</>;
};
