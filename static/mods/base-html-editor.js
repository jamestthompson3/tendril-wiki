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
  #machine;
  constructor(element) {
    this.element = element;
    // plain-text tag array
    this.content = htmlToText(this.element);
    this.bc = new BroadcastChannel(`tendril-wiki${location.pathname}`);
    this.#machine = new StateMachine({
      ...stateChart,
      actions: {
        showErrors: () => {
          const errorMsg = document.querySelector(this.errorSelector);
          errorMsg.classList.remove("hidden");
        },
        hideErrors: () => {
          const errorMsg = document.querySelector(this.errorSelector);
          errorMsg.classList.add("hidden");
        },
      },
    });
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
  change = (e) => {
    this.content = e.target.value;
    if (e.target.checkValidity()) {
      if (this.#machine.state === "error") {
        this.#machine.send("RESET");
      }
      this.bc.postMessage({
        type: "SAVE",
        data: { id: this.id, content: this.content },
      });
    } else {
      this.#machine.send("ERROR");
    }
  };
}
