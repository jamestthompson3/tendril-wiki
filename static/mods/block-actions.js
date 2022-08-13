import { textToHtml, htmlToText } from "./parsing.js";

let currentFocusedElement;

export function addBlock(indentationLevel) {
  const textblock = document.createElement("textarea");
  if (indentationLevel) {
    textblock.dataset.indent = indentationLevel;
  }
  setupTextblockListeners(textblock);
  // insert the new block directly after the current block
  const { parentNode, nextSibling } = this;
  parentNode.insertBefore(textblock, nextSibling);
  setAsFocused(textblock);
}

export function deleteBlock(el) {
  el.remove();
}

export function saveBlock(el, type) {
  el.dataset.type = type;
}

export function setAsFocused(el) {
  if (currentFocusedElement) {
    currentFocusedElement.blur();
  }
  currentFocusedElement = el;
  el.focus();
  el.selectionStart = el.textContent?.length;
  updateInputHeight(el);
}

export function updateInputHeight(el) {
  // Let's handle text overflows
  const scrollHeight = el.scrollHeight;
  const clientHeight = el.clientHeight;
  const scrollTop = el.scrollTop;
  const heightDiff = scrollHeight - scrollTop - clientHeight;
  if (heightDiff > 0 || scrollHeight > clientHeight) {
    el.style.height = `${scrollHeight}px`;
  }
}

export function setupViewer(e) {
  const div = document.createElement("div");
  div.addEventListener("click", setupEditor);
  div.innerHTML = textToHtml(e.target.value);
  div.classList.add("text-block");
  if (this.value !== "" && attributesNotSet(e.target)) {
    saveBlock(div, "text/wikitext");
  }
  for (const datapoint in this.dataset) {
    div.dataset[datapoint] = this.dataset[datapoint];
  }
  e.target.replaceWith(div);
}

export function setupEditor(e) {
  // don't try to edit the block when we're clicking a link
  if (e.target.nodeName === "A") return;
  const textblock = document.createElement("textarea");
  htmlToText(this);
  textblock.textContent = this.textContent;
  for (const datapoint in this.dataset) {
    textblock.dataset[datapoint] = this.dataset[datapoint];
  }
  setupTextblockListeners(textblock);
  this.replaceWith(textblock);
  setAsFocused(textblock);
}

function attributesNotSet(el) {
  if (el.dataset.type) {
    return false;
  }
  return true;
}

export function handleInput(e) {
  switch (e.key) {
    case "Backspace": {
      // TODO handle indenting back one level.
      if (e.target.value === "") {
        deleteBlock(e.target);
        break;
      }
      break;
    }
    case "Enter": {
      if (!e.shiftKey) {
        const indentation = this.dataset.indent;
        addBlock.bind(this)(indentation && Number(indentation));
        break;
      }
      break;
    }
    default:
      break;
  }
}

export function handleKeydown(e) {
  if (e.key === "Tab") {
    e.preventDefault();
    const indentation = this.dataset.indent;
    if (indentation) {
      // Max indent is 3 levels, min is 0
      const indentationLevel = e.shiftKey
        ? Math.max(Number(indentation) - 1, 0)
        : Math.min(Number(indentation) + 1, 3);
      this.dataset.indent = indentationLevel;
    } else if (!e.shiftKey) {
      this.dataset.indent = 1;
    }
  } else {
    updateInputHeight(this);
  }
}

function setupTextblockListeners(textblock) {
  textblock.addEventListener("blur", setupViewer);
  textblock.addEventListener("keyup", handleInput);
  textblock.addEventListener("keydown", handleKeydown);
}
