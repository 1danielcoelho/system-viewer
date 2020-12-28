// Fetches some file at `url` and relays the data as text to the wasm module
export function load_text(url, content_type) {
  return fetch(url).then((response) =>
    response.text().then((text) => {
      window.wasm_module.receive_text(url, content_type, text);
    })
  );
}

// Fetches some file at `url` and relays the data as bytes to the wasm module
export function load_bytes(url, content_type) {
  return fetch(url).then((response) =>
    response.arrayBuffer().then((buffer) => {
      window.wasm_module.receive_bytes(
        url,
        content_type,
        new Uint8Array(buffer)
      );
    })
  );
}

// Opens a file prompt for a json file and relays the data as text to the wasm module
export function prompt_for_text_file(content_type) {
  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = ".json";

  fileInput.addEventListener("change", (e) => {
    if (fileInput.files && fileInput.files[0]) {
      const file = fileInput.files[0];

      const reader = new FileReader();
      reader.onload = async (loadEvent) => {
        const data = loadEvent.target.result;

        window.wasm_module.receive_text(file.name, content_type, data);
      };

      reader.readAsText(file);
    }
  });

  fileInput.click();
}
