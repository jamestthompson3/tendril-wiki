import { setAsFocused } from "./block-actions.js";

export class TagEditor {
  constructor(element) {
    this.element = element;
    this.id = "tag";
    // plain-text tag array
    this.content = Array.from(element.querySelectorAll("a")).map((el) =>
      el.innerText.replace("#", "")
    );
    element.addEventListener("click", this.handleClick);
    this.bc = new BroadcastChannel(`tendril-wiki${location.pathname}`);
    this.bc.postMessage({
      type: "REGISTER",
      data: { id: this.id, content: this.content },
    });
  }
  handleClick = (e) => {
    if (e.target.nodeName === "A") return;
    const textblock = document.createElement("input");
    textblock.type = "text";
    textblock.value = this.content;
    textblock.addEventListener("blur", this.setupTagViewer);
    textblock.addEventListener("change", this.handleChange);
    this.element.replaceWith(textblock);
    this.element = textblock;
    setAsFocused(textblock);
  };
  setupTagViewer = () => {
    const container = document.createElement("div");
    const list = document.createElement("ul");
    this.content.forEach((tag) => {
      const trimmed = tag.trim();
      const child = document.createElement("li");
      child.innerHTML = `<a href="${trimmed}">#${trimmed}</a>`;
      list.appendChild(child);
    });
    container.appendChild(list);
    container.classList.add("tags");
    container.addEventListener("click", this.setupTagEditor);
    this.element.replaceWith(container);
    this.element = container;
  };
  handleChange = (e) => {
    this.content = e.target.value.split(",");
    this.bc.postMessage({
      type: "SAVE",
      data: { id: this.id, content: this.content },
    });
  };
}
