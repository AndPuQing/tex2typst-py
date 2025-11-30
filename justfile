dist:
    yarn install
    npx esbuild entry.js --bundle --outfile=js/tex2typst.bundle.js --format=iife --minify

build: dist
    maturin build

dev: dist
    maturin develop
