import { htmlToText } from "./parsing.js";
import { StateMachine, isIOS, normalizePunction } from "./utils.js";

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
    element.addEventListener("blur", () => {
      if (document.getElementById("autocomplete-menu")) return;
      this.setupViewer(this.eventTarget);
    });
    element.addEventListener("keyup", this.handleInput);
    element.addEventListener("keydown", this.handleKeydown);
    element.addEventListener("paste", this.detectImagePaste);
    element.addEventListener("change", this.change);
    if (isIOS()) {
      normalizePunction(element);
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
