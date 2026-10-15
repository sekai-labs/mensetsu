import { writeFileSync } from "node:fs";
import openapiTS, { astToString, COMMENT_HEADER } from "openapi-typescript";

const ast = await openapiTS(new URL("./openapi.json", import.meta.url));
const output = astToString(ast);

if (output.includes(COMMENT_HEADER.trim())) {
  throw new Error("openapi-typescript emitted its banner; generator contract broken");
}
writeFileSync(new URL("./src/generated/api.ts", import.meta.url), output);
console.log("openapi.json -> server/api/contracts/src/generated/api.ts");

const pkgOutPath = new URL("../../../packages/contracts/src/generated/api.ts", import.meta.url);
writeFileSync(pkgOutPath, output);
console.log("openapi.json -> packages/contracts/src/generated/api.ts");
