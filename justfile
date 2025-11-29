dist:
    npx esbuild entry.js --bundle --outfile=dist/tex2typst.bundle.js --format=iife --minify
build:dist
    maturin build
dev:dist
    maturin develop