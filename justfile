build:
    # Bundle the JS code using esbuild
    npx esbuild entry.js --bundle --outfile=dist/tex2typst.bundle.js --format=iife
    
    maturin build