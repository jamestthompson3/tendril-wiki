import { StateMachine } from "./utils.js";
import caretPos from "/static/vendors/caretposition.js";

const stateChart = {
  initial: "idle",
  states: {
    idle: {
      on: { "[": "ready" },
    },
    ready: {
      on: { "[": "completing" },
    },
    completing: {
      on: { "]": "finishing" },
    },
    finishing: {
      on: { "]": "idle" },
    },
  },
};

const machine = new StateMachine(stateChart);
export function autocomplete() {
  document.querySelector("textarea").addEventListener("keyup", function (e) {
    if (e.key === "[") {
      machine.send("[");
    }
    if (e.key === "]") {
      machine.send("]");
    }
    if (machine.state === "completing") {
      caretPos();
      const caret = getCaretCoordinates(this, this.selectionEnd);
      const suggestions = document.createElement("div");
      suggestions.style.position = "absolute";
      const HEIGHT = 100;
      console.log({
        scrollWidth: this.scrollWidth,
        caretLeft: caret.left,
        obj: this,
      });
      suggestions.style.top = `${this.scrollHeight + HEIGHT * 2}px`;

      suggestions.style.left = `${this.offsetLeft + caret.left}px`;

      suggestions.style.height = `${HEIGHT}px`;

      suggestions.style.backgroundColor = "blue";

      document.body.appendChild(suggestions);

      const opts = ["cat", "dog,", "dolphin"];

      const list = document.createElement("ul");

      suggestions.appendChild(list);

      opts.forEach((opt) => {
        const item = document.createElement("li");

        item.textContent = opt;

        list.appendChild(item);
      });
    }
  });
}
