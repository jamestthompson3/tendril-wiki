import { textToHtml } from "./parsing.js";
import { HTMLEditor } from "./base-html-editor.js";
import { setAsFocused } from "./block-actions.js";

export class TitleEditor extends HTMLEditor {
  constructor(element) {
    super(element);
    element.addEventListener("click", this.setupEditor);
    this.id = "title";
    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.content },
    });
  }
  setupViewer = (e) => {
    const html = textToHtml(e.target.value);
    const el = document.createElement("h1");
    el.classList.add("title");
    el.innerHTML = html;
    el.addEventListener("click", this.setupEditor);
    for (const datapoint in this.element.dataset) {
      el.dataset[datapoint] = this.element.dataset[datapoint];
    }
    this.element.replaceWith(el);
    this.element = el;
  };
  setupEditor = () => {
    const textblock = document.createElement("input");
    textblock.type = "text";
    textblock.value = this.content;
    for (const datapoint in this.element.dataset) {
      textblock.dataset[datapoint] = this.element.dataset[datapoint];
    }
    textblock.classList.add("title");
    this.setupTextblockListeners(textblock);
    this.element.replaceWith(textblock);
    setAsFocused(textblock);
    this.element = textblock;
  };
}
