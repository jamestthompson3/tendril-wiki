import { htmlToText } from "./parsing.js";

export class HTMLEditor {
  constructor(element) {
    this.element = element;
    // plain-text tag array
    this.content = htmlToText(this.element);
  }
  setupTextblockListeners = (element) => {
    element.addEventListener("blur", this.setupViewer);
    element.addEventListener("keyup", this.handleInput);
    element.addEventListener("keydown", this.handleKeydown);
    element.addEventListener("paste", this.detectImagePaste);
    element.addEventListener("change", this.change);
  };
  detectImagePaste = () => {};
  handleKeydown = () => {};
  handleInput = () => {};
}
