import { writeFileSync } from "node:fs";
import openapiTS, { astToString, COMMENT_HEADER } from "openapi-typescript";

const openapiPath = new URL("../openapi.json", import.meta.url);
const outPath = new URL("../src/generated/api.ts", import.meta.url);

const ast = await openapiTS(openapiPath);
const output = astToString(ast);

if (output.includes(COMMENT_HEADER.trim())) {
  throw new Error("openapi-typescript emitted its banner; generator contract broken");
}

writeFileSync(outPath, output);
console.log(`Generated ${outPath.pathname} from ${openapiPath.pathname}`);
