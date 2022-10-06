import { textToHtml } from "./parsing.js";
import { HTMLEditor } from "./base-html-editor.js";
import { setAsFocused } from "./block-actions.js";

export class TitleEditor extends HTMLEditor {
  constructor(element) {
    super(element);
    element.addEventListener("click", this.setupEditor);
    this.errorMsg =
      "Titles cannot contain special characters other than _+—. Titles must not be blank.";
    this.id = "title";
    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.content },
    });
    fetch("/titles")
      .then((res) => res.json())
      .then((titles) => {
        this.titles = titles.map((t) => t.toLowerCase());
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
    textblock.minLength = 1;
    textblock.setAttribute("pattern", "([a-zA-Z0-9-_+—]\\s?)+");
    for (const datapoint in this.element.dataset) {
      textblock.dataset[datapoint] = this.element.dataset[datapoint];
    }
    textblock.classList.add("title");
    this.setupTextblockListeners(textblock);
    this.element.replaceWith(textblock);
    setAsFocused(textblock);
    this.element = textblock;
  };

  change = (e) => {
    this.content = e.target.value;
    if (this.titles.includes(e.target.value.toLowerCase())) {
      this.errorMsg = "A note by this title already exists!";
      this.machine.send("ERROR");
      return;
    }
    if (e.target.checkValidity()) {
      if (this.machine.state === "error") {
        this.errorMsg =
          "Titles cannot contain special characters other than _+—. Titles must not be blank.";
        this.machine.send("RESET");
      }
      this.bc.postMessage({
        type: "SAVE",
        data: { id: this.id, content: this.content },
      });
    } else {
      this.machine.send("ERROR");
    }
  };
}
