import { TagEditor } from "./mods/tag-editor.js";
import { TitleEditor } from "./mods/title-editor.js";
import { BlockEditor } from "./mods/block-editor.js";
import { MetaDataEditor } from "./mods/metadata-editor.js";
import { ComponentRegister } from "./mods/component-register.js";

new ComponentRegister();

(function () {
  const content = document.getElementById("content-block");
  content.querySelectorAll(".text-block").forEach(function (el) {
    new BlockEditor(el);
  });

  const title = document.querySelector(".title");
  new TitleEditor(title);

  const tags = document.querySelector(".tags");
  new TagEditor(tags);

  const metadata = document.getElementById("metadata");
  new MetaDataEditor(metadata);
})();

/* TESTING */
const shouldRunTests = false;

(async () => {
  if (shouldRunTests) {
    // import testing module for side effects
    await import("/static/tests.js");
  }
})();
