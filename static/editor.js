import { setupEditor, setupTagEditor } from "./mods/block-actions.js";

(function () {
  const content = document.getElementById("content-block");
  content.querySelectorAll(".text-block").forEach(function (el) {
    el.addEventListener("click", setupEditor("text-block"));
  });
  const title = document.querySelector(".title");
  title.addEventListener("click", setupEditor("title"));
  const tags = document.querySelector(".tags");
  tags.addEventListener("click", setupTagEditor);
})();

/* TESTING */
const shouldRunTests = false;

(async () => {
  if (shouldRunTests) {
    // import testing module for side effects
    await import("/static/tests.js");
  }
})();
