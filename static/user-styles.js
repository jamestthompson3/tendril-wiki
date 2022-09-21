import { updateInputHeight } from "./mods/block-actions.js";

const editor = document.getElementById("body");

editor.addEventListener("keydown", handleKeydown);

function handleKeydown() {
  updateInputHeight(editor);
}
