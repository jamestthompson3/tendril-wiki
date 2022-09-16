import { textToHtml } from "./parsing.js";
import { HTMLEditor } from "./base-html-editor.js";
import { nanoid } from "./utils.js";
import {
  setAsFocused,
  updateInputHeight,
  deleteBlock,
} from "./block-actions.js";

export class BlockEditor extends HTMLEditor {
  constructor(element) {
    super(element);
    this.id = `block@${nanoid()}`;
    if (element.nodeName === "TEXTAREA") {
      this.setupTextblockListeners(element);
    } else {
      element.addEventListener("click", this.setupEditor);
    }

    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.content },
    });
  }
  setupViewer = (e) => {
    const html = textToHtml(e.target.value);
    const el = document.createElement("div");
    el.innerHTML = html;
    el.classList.add("text-block");
    el.addEventListener("click", this.setupEditor);
    for (const datapoint in this.element.dataset) {
      el.dataset[datapoint] = this.element.dataset[datapoint];
    }
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
    textblock.classList.add("text-block");
    this.setupTextblockListeners(textblock);
    this.element.replaceWith(textblock);
    setAsFocused(textblock);
    this.element = textblock;
  };
  handleInput = (e) => {
    switch (e.key) {
      case "Backspace": {
        if (e.target.value === "" && e.target.parentNode.children.length > 1) {
          deleteBlock(e.target);
          this.bc.postMessage({ type: "UNREGISTER", data: this.id });
          this.bc.postMessage({ type: "SAVE" });
          break;
        }
        break;
      }
      case "Enter": {
        if (!e.shiftKey) {
          this.element.value = this.element.value.slice(
            0,
            this.element.value.length - 1
          );
          const indentation = this.element.dataset.indent;
          this.addBlock(indentation && Number(indentation));
          break;
        }
        break;
      }
      default:
        break;
    }
  };

  handleKeydown = (e) => {
    if (e.key === "Tab") {
      // TODO: Figure out if I want to deal with indentation like an outliner.
      // e.preventDefault();
      // const indentation = this.dataset.indent;
      // if (indentation) {
      //   // Max indent is 3 levels, min is 0
      //   const indentationLevel = e.shiftKey
      //     ? Math.max(Number(indentation) - 1, 0)
      //     : Math.min(Number(indentation) + 1, 3);
      //   this.dataset.indent = indentationLevel;
      // } else if (!e.shiftKey) {
      //   this.dataset.indent = 1;
      // }
    } else {
      updateInputHeight(this);
    }
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
          `image-${new Date().valueOf()}.${extension}`
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
    new BlockEditor(textblock);
    // insert the new block directly after the current block
    const { parentNode, nextSibling } = this.element;
    parentNode.insertBefore(textblock, nextSibling);
    setAsFocused(textblock);
  };
}
