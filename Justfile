compile:
    cargo build --release

compile-web out-dir="web/pkg":
    wasm-pack build --target web --release --out-name json-sort-rs --out-dir {{ out-dir }}
    npm pkg set name=json-sort-rs --prefix {{ out-dir }}

# Build and publish the WebAssembly package to npm.
publish-npm: (compile-web)
    cd web/pkg && npm pkg get name version && npm publish

# Serve the WebAssembly demo with Caddy.
serve port="8080": compile-web
    echo "Web server available at http://127.0.0.1:{{ port }}"
    caddy file-server --root web --listen ":{{ port }}"
