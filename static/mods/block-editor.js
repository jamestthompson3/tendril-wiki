import { textToHtml } from "./parsing.js";
import { moveCaretToEnd, moveCaretToStart } from "./dom.js";
import { HTMLEditor } from "./base-html-editor.js";
import { nanoid } from "./utils.js";
import {
  setAsFocused,
  updateInputHeight,
  deleteBlock,
} from "./block-actions.js";
import {
  autocomplete,
  autocompleteState,
  removeAutoCompleteMenu,
} from "./autocomplete.js";

export class BlockEditor extends HTMLEditor {
  #shouldStopExecution;
  constructor(element, parent) {
    super(element);
    this.id = `block@${nanoid()}`;
    this.indent = parseInt(element.dataset.indent || 0, 10);
    if (element.nodeName === "TEXTAREA") {
      this.setupTextblockListeners(element);
    } else {
      element.addEventListener("click", this.setupEditor);
    }

    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.prepareContent(this.content), parent },
    });
  }
  setupViewer = (element) => {
    removeAutoCompleteMenu();
    const html = textToHtml(element.value);
    const el = document.createElement("div");
    el.innerHTML = html;
    el.classList.add("text-block");
    el.addEventListener("click", this.setupEditor);
    el.addEventListener("keyup", (e) => {
      if (e.key === "Enter") {
        this.setupEditor(e);
      }
    });
    for (const datapoint in this.element.dataset) {
      el.dataset[datapoint] = this.element.dataset[datapoint];
    }
    el.tabIndex = 0;
    this.element.replaceWith(el);
    this.element = el;
  };
  setupEditor = (e) => {
    // don't try to edit the block when we're clicking a link
    if (e.target.nodeName === "A") return;
    const textblock = document.createElement("textarea");
    textblock.textContent = this.content;
    for (const datapoint in this.element.dataset) {
      textblock.dataset[datapoint] = this.element.dataset[datapoint];
    }
    textblock.setAttribute("spellcheck", true);
    textblock.classList.add("text-block");
    this.setupTextblockListeners(textblock);
    this.element.replaceWith(textblock);
    setAsFocused(textblock);
    moveCaretToEnd(textblock);
    this.element = textblock;
  };

  handleKeydown = (e) => {
    autocomplete(e);
    this.#shouldStopExecution = autocompleteState() === "completing";
    switch (e.key) {
      case "Tab": {
        if (!this.#shouldStopExecution) {
          e.preventDefault();
          if (this.indent) {
            // Max indent is 3 levels, min is 0
            const indentationLevel = e.shiftKey
              ? Math.max(this.indent - 1, 0)
              : Math.min(this.indent + 1, 3);
            e.target.dataset.indent = this.indent = indentationLevel;
          } else if (!e.shiftKey) {
            e.target.dataset.indent = this.indent = 1;
          }
          this.change(e);
        }
        break;
      }
      case "Backspace": {
        if (e.target.value === "" && e.target.parentNode.children.length > 1) {
          deleteBlock(e.target);
          this.bc.postMessage({ type: "UNREGISTER", data: this.id });
          this.bc.postMessage({ type: "SAVE" });
          break;
        }
        break;
      }
      case "Escape": {
        this.element.blur();
        break;
      }
      case "Home": {
        moveCaretToStart(this.element);
        break;
      }
      case "Enter": {
        if (!e.shiftKey && !this.#shouldStopExecution) {
          e.preventDefault();
          const indentation = this.indent;
          this.addBlock(indentation);
          if (this.element.type === "TEXTAREA") {
            this.setupViewer(this.element);
          }
          break;
        }
        break;
      }
      default: {
        break;
      }
    }
    updateInputHeight(e.target);
  };
  detectImagePaste = (e) => {
    const items = (e.clipboardData || e.originale.clipboardData).items;
    for (let index in items) {
      const item = items[index];
      if (item.kind === "file") {
        // we need to get the filename, and `getAsFile` clobbers this with a generic name
        // so we can just use FormData here.
        const formData = new FormData();
        const file = item.getAsFile();
        const extension = file.type.split("image/").find(Boolean);
        formData.append(
          "file",
          item.getAsFile(),
          `image-${new Date().valueOf()}.${extension}`,
        );
        const blob = formData.get("file");
        fetch("/files", {
          method: "POST",
          headers: {
            "Content-Type": "application/octet-stream",
            Filename: `${blob.name}`,
          },
          body: blob,
        })
          .then(() => {
            // TODO figure out alt text...
            e.target.value += `${window.location.origin}/files/${blob.name}`;
            e.target.dispatchEvent(new Event("change"));
          })
          .catch((e) => {
            console.error(e);
            e.target.value += "Failed to upload image";
          });
      }
    }
  };
  addBlock = (indentationLevel) => {
    const textblock = document.createElement("textarea");
    if (indentationLevel) {
      textblock.dataset.indent = indentationLevel;
    }
    textblock.setAttribute("spellcheck", true);
    new BlockEditor(textblock, this.id);
    // insert the new block directly after the current block
    const { parentNode, nextSibling } = this.element;
    parentNode.insertBefore(textblock, nextSibling);
    setAsFocused(textblock);
  };
  change = (e) => {
    this.content = e.target.value;
    // Check to see if we're about to commit invalid syntax with unclosed link brackets
    // This occurs when we insert an option from autocomplete.
    const words = e.target.value.split(" ");
    const lastWord = words[words.length - 1];
    if (lastWord.startsWith("[[") && !lastWord.endsWith("]]")) return;
    if (autocompleteState() === "completing") return;
    this.bc.postMessage({
      type: "SAVE",
      data: {
        id: this.id,
        content: this.prepareContent(),
      },
    });
  };
  prepareContent = () => {
    const targetLength = this.indent + this.content.length;
    return this.content.padStart(targetLength, String.fromCharCode(9));
  };
}
