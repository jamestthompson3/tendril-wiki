import { textToHtml, htmlToText } from "./utils/parsing.js";

(function () {
  let currentFocusedElement;
  const content = document.getElementById("content-block");
  content.querySelectorAll(".text-block").forEach(function (el) {
    el.addEventListener("click", setupEditor);
  });

  function setupEditor() {
    const textArea = document.createElement("textarea");
    textArea.textContent = this.textContent;
    textArea.addEventListener("keyup", handleInput);
    textArea.addEventListener("keydown", (e) => updateInputHeight(e.target));
    textArea.addEventListener("blur", setupViewer);
    for (const datapoint in this.dataset) {
      textArea.dataset[datapoint] = this.dataset[datapoint];
    }
    this.replaceWith(textArea);
    setAsFocused(textArea);
  }
  function handleInput(e) {
    switch (e.key) {
      case "Backspace": {
        if (e.target.value === "") {
          deleteBlock(e.target);
          break;
        }
        break;
      }
      case "Enter": {
        if (!e.shiftKey) {
          addBlock(e);
          break;
        }
        break;
      }
      default:
        break;
    }
  }
  function setupViewer(e) {
    const div = document.createElement("div");
    div.addEventListener("click", setupEditor);
    // div.textContent = e.target.value;
    div.innerHTML = textToHtml(e.target.value);
    div.classList.add("text-block");
    if (this.value !== "" && attributesNotSet(e.target)) {
      saveBlock(div, "text/wikitext");
    }
    e.target.replaceWith(div);
  }
  function addBlock(e) {
    const textblock = document.createElement("textarea");
    textblock.addEventListener("blur", setupViewer);
    textblock.addEventListener("keyup", handleInput);
    content.appendChild(textblock);
    setAsFocused(textblock);
  }
  function setAsFocused(el) {
    if (currentFocusedElement) {
      currentFocusedElement.blur();
    }
    currentFocusedElement = el;
    el.focus();
    el.selectionStart = el.textContent?.length;
    updateInputHeight(el);
  }
  function deleteBlock(el) {
    el.remove();
  }
  function updateInputHeight(el) {
    // Let's handle text overflows
    const scrollHeight = el.scrollHeight;
    const clientHeight = el.clientHeight;
    const scrollTop = el.scrollTop;
    const heightDiff = scrollHeight - scrollTop - clientHeight;
    if (heightDiff > 0 || scrollHeight > clientHeight) {
      el.style.height = `${scrollHeight}px`;
    }
  }
  function saveBlock(el, type) {
    el.dataset.type = type;
  }
  function attributesNotSet(el) {
    if (el.getAttribute("data-type")) {
      return false;
    }
    return true;
  }
})();

/* TESTING */
const shouldRunTests = false;

(async () => {
  if (shouldRunTests) {
    // import testing module for side effects
    await import("/static/tests.js");
  }
})();
