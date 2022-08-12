import { setupEditor } from "./mods/block-actions.js";

(function () {
  const content = document.getElementById("content-block");
  content.querySelectorAll(".text-block").forEach(function (el) {
    el.addEventListener("click", setupEditor);
  });
})();

/* TESTING */
const shouldRunTests = true;

(async () => {
  if (shouldRunTests) {
    // import testing module for side effects
    await import("/static/tests.js");
  }
})();
