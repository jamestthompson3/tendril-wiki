import { htmlToText } from "./parsing.js";
import { StateMachine } from "./utils.js";

const stateChart = {
  initial: "idle",
  states: {
    idle: {
      on: {
        ERROR: { target: "error", actions: ["showErrors"] },
      },
    },
    error: {
      on: {
        RESET: { target: "idle", actions: ["hideErrors"] },
      },
    },
  },
};

export class HTMLEditor {
  machine;
  constructor(element) {
    this.element = element;
    this.eventTarget = undefined;
    // plain-text tag array
    this.content = htmlToText(this.element);
    this.bc = new BroadcastChannel(`tendril-wiki${location.pathname}`);
    this.machine = new StateMachine({
      ...stateChart,
      actions: {
        showErrors: () => {
          const errorMsg = document.querySelector(".error-msg");
          errorMsg.classList.remove("hidden");
          errorMsg.textContent = this.errorMsg;
        },
        hideErrors: () => {
          const errorMsg = document.querySelector(".error-msg");
          errorMsg.classList.add("hidden");
          errorMsg.textContent = "";
        },
      },
    });
  }
  setupTextblockListeners = (element) => {
    this.eventTarget = element;
    document.addEventListener("click", this.handleOutsideClick);
    element.addEventListener("keyup", this.handleInput);
    element.addEventListener("keydown", this.handleKeydown);
    element.addEventListener("paste", this.detectImagePaste);
    element.addEventListener("change", this.change);
  };
  handleOutsideClick = (e) => {
    if (!e.target.nextSibling || e.target === this.eventTarget) return;
    let currentTag = e.target;
    let sibling = false;
    while (currentTag.nextSibling) {
      if (currentTag.nextSibling === this.eventTarget) {
        sibling = true;
      }
      currentTag = currentTag.nextSibling;
    }
    if (!sibling) {
      document.removeEventListener("click", this.handleOutsideClick);
      this.setupViewer(this.eventTarget);
    }
  };
  detectImagePaste = () => {};
  handleKeydown = () => {};
  handleInput = () => {};
  change = (e) => {
    this.content = e.target.value;
    if (e.target.checkValidity()) {
      if (this.machine.state === "error") {
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
