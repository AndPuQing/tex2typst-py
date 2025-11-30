import { tex2typst, typst2tex } from "tex2typst";

// Export both conversion functions to the global scope
globalThis.tex2typst = tex2typst;
globalThis.typst2tex = typst2tex;
