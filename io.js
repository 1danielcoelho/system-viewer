// Fetches some file at `url` and relays the data as text to the wasm module
export function fetch_text(url, content_type) {
  console.log(
    "JS fetching text from url '",
    url,
    "' with content_type '" + content_type + "'"
  );
  return fetch(url).then((response) =>
    response.text().then((text) => {
      window.wasm_module.receive_text(
        // Strip path information when returning, as wasm always assumes we can't know
        // the file path (due to how the file path is stripped when loading files via prompt)
        url.replace(/^.*[\\\/]/, ""),
        content_type,
        text
      );
    })
  );
}

// Fetches some file at `url` and relays the data as bytes to the wasm module
export function fetch_bytes(url, content_type) {
  console.log(
    "JS fetching bytes from url '",
    url,
    "' with content_type '" + content_type + "'"
  );
  return fetch(url).then((response) =>
    response.arrayBuffer().then((buffer) => {
      window.wasm_module.receive_bytes(
        // Strip path information when returning, as wasm always assumes we can't know
        // the file path (due to how the file path is stripped when loading files via prompt)
        url.replace(/^.*[\\\/]/, ""),
        content_type,
        new Uint8Array(buffer)
      );
    })
  );
}

// Opens a file prompt for a json file and relays the data as text to the wasm module
export function prompt_for_text_file(content_type, extension) {
  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = extension;

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

export function prompt_for_bytes_file(content_type, extension) {
  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = extension;

  fileInput.addEventListener("change", (e) => {
    if (fileInput.files && fileInput.files[0]) {
      const file = fileInput.files[0];

      const reader = new FileReader();
      reader.onload = async (loadEvent) => {
        const data = loadEvent.target.result;

        window.wasm_module.receive_bytes(
          file.name,
          content_type,
          new Uint8Array(data)
        );
      };

      reader.readAsArrayBuffer(file);
    }
  });

  fileInput.click();
}
