import { setAsFocused, updateInputHeight } from "./block-actions.js";
import { htmlToText, textToHtml } from "./parsing.js";
import { isIOS, normalizePunctuation } from "./utils.js";
import { moveCaretToEnd } from "./dom.js";

export class MetaDataEditor {
  constructor(element) {
    this.element = element;
    this.id = "metadata";
    const v = Array.from(element.querySelectorAll("dd"));
    const k = Array.from(element.querySelectorAll("dt"));
    this.content = {};

    k.forEach((key, idx) => {
      this.content[key.textContent] =
        key.textContent === "isbn" ? v[idx].textContent : htmlToText(v[idx]);
    });
    element.addEventListener("click", this.handleClick);

    this.bc = new BroadcastChannel(`tendril-wiki${location.pathname}`);
    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.content },
    });
  }

  parse = (str) =>
    str.split("\n").reduce((obj, line) => {
      const parts = line.split(":");
      const key = parts[0];
      const trimmed = parts.slice(1).join(":").trim();
      obj[key] = trimmed;
      return obj;
    }, {});

  handleClick = (e) => {
    if (e.target.nodeName === "A") return;
    const textblock = document.createElement("textarea");
    textblock.value = Object.entries(this.content)
      .map(([key, value]) => `${key}:${value}`)
      .join("\n");
    textblock.addEventListener("blur", this.setupViewer);
    textblock.addEventListener("change", this.handleChange);
    textblock.addEventListener("keyup", () => updateInputHeight(textblock));
    if (isIOS()) {
      normalizePunctuation(textblock);
    }
    this.element.replaceWith(textblock);
    setAsFocused(textblock);
    moveCaretToEnd(textblock);
    this.element = textblock;
  };
  setupViewer = () => {
    const container = document.createElement("dl");

    Object.entries(this.content).forEach(([key, value]) => {
      const trimmed = value.trim();
      const term = document.createElement("dt");
      term.textContent = key;
      const description = document.createElement("dd");
      description.innerHTML = textToHtml(trimmed);
      // TODO: Figure out how to handle things like auto-inserting cover from ISBN
      container.append(term, description);
    });
    container.id = "metadata";
    container.addEventListener("click", this.handleClick);
    this.element.replaceWith(container);
    this.element = container;
  };
  handleChange = (e) => {
    this.content = this.parse(e.target.value);
    this.bc.postMessage({
      type: "SAVE",
      data: { id: this.id, content: this.content },
    });
  };
}
