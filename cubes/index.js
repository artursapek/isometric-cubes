import('./styles.css');

import('./pkg')
  .then(async (wasm) => {
    await wasm.default();  // Initialize the WASM module
    wasm.start();          // Start the application
  })
  .catch(console.error);
