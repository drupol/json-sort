compile:
    cargo build --release

compile-web out-dir="web/pkg":
    wasm-pack build --target web --out-dir {{ out-dir }}

# Serve the WebAssembly demo with Caddy.
serve port="8080": compile-web
    echo "Web server available at http://127.0.0.1:{{ port }}"
    caddy file-server --root web --listen ":{{ port }}"
