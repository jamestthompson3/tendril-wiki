import { HTMLEditor } from "./base-html-editor.js";
import { setAsFocused } from "./block-actions.js";

export class TitleEditor extends HTMLEditor {
  #titles;
  constructor(element) {
    super(element);
    element.addEventListener("click", this.setupEditor);
    this.errorMsg =
      "Titles cannot contain special characters other than _+—. Titles must not be blank.";
    this.id = "title";
    this.content = element.textContent;
    this.validator = /^[A-Za-z0-9-\s_\+]+/;
    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.content },
    });
  }
  setupViewer = (element) => {
    const el = document.createElement("h1");
    el.classList.add("title");
    el.textContent = element.value;
    el.addEventListener("click", this.setupEditor);
    for (const datapoint in this.element.dataset) {
      el.dataset[datapoint] = this.element.dataset[datapoint];
    }
    this.element.replaceWith(el);
    this.element = el;
  };
  setupEditor = () => {
    const template = document.getElementById("title-editor");
    const textblock = template.content.cloneNode(true).querySelector(".title");
    textblock.value = this.content;
    this.setupTextblockListeners(textblock);
    this.element.replaceWith(textblock);
    setAsFocused(textblock);
    this.element = textblock;
  };

  change = async (e) => {
    if (!this.#titles) {
      this.#titles = await fetch("/titles")
        .then((res) => res.json())
        .then((titles) => titles.map((t) => t.toLowerCase()));
    }
    if (this.#titles.includes(e.target.value.toLowerCase())) {
      this.errorMsg = "A note by this title already exists!";
      this.machine.send("ERROR");
    }
    if (!this.validator.test(e.target.value)) {
      this.errorMsg =
        "Titles cannot contain special characters other than _+—. Titles must not be blank.";
      this.machine.send("ERROR");
      return;
    }

    this.machine.send("RESET");
    this.content = e.target.value;
    this.bc.postMessage({
      type: "SAVE",
      data: { id: this.id, content: this.content },
    });
  };
}
