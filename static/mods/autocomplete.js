import { assign, StateMachine } from "./utils.js";
import caretPos from "/static/vendors/caretposition.js";

const stateChart = {
  initial: "idle",
  context: {
    suggestions: undefined,
  },
  states: {
    idle: {
      on: { "[": "ready" },
    },
    ready: {
      on: {
        "[": {
          target: "setup",
          actions: [setupMenu],
        },
        reset: {
          target: "idle",
          actions: [teardownMenu, assign(() => ({ suggestions: undefined }))],
        },
      },
    },
    setup: {
      on: {
        done: {
          target: "completing",
          actions: [assign((_, payload) => ({ suggestions: payload }))],
        },
        reset: {
          target: "idle",
          actions: [teardownMenu, assign(() => ({ suggestions: undefined }))],
        },
      },
    },
    completing: {
      on: {
        "]": {
          target: "idle",
          actions: [teardownMenu, assign(() => ({ suggestions: undefined }))],
        },
        reset: {
          target: "idle",
          actions: [teardownMenu, assign(() => ({ suggestions: undefined }))],
        },
      },
    },
  },
};

const machine = new StateMachine(stateChart);
export function autocomplete(e) {
  const { suggestions } = machine.context();
  if (e.key === "[") {
    machine.send("[", e);
  }
  if (e.key === "]") {
    machine.send("]", e);
  }
  if (e.key === "ArrowDown") {
    if (machine.state === "completing" && suggestions) {
      const candidate = suggestions.querySelector('button[data-idx="0"]');
      candidate.focus();
    }
  }
}

function setupMenu(e) {
  caretPos();
  const caret = getCaretCoordinates(e.target, e.target.selectionEnd);
  const suggestions = document.createElement("div");
  suggestions.classList.add("autocomplete-menu");
  suggestions.style.position = "absolute";
  const HEIGHT = 95;
  suggestions.style.top = `${e.target.scrollHeight + HEIGHT * 2}px`;

  suggestions.style.left = `${e.target.offsetLeft + caret.left}px`;

  const opts = ["cat", "dog", "dolphin"];

  const list = document.createElement("ul");

  opts.forEach((opt, idx) => {
    const item = document.createElement("li");
    const innerButton = document.createElement("button");
    innerButton.innerText = opt;
    innerButton.setAttribute("data-idx", idx);
    innerButton.addEventListener("click", handleClick);
    item.appendChild(innerButton);
    list.appendChild(item);
  });
  suggestions.appendChild(list);
  e.target.parentElement.appendChild(suggestions);
  machine.send("done", suggestions);
}

function handleClick(e) {
  console.log(e.target.innerText);
}
function teardownMenu() {
  if (machine.context().suggestions) {
    machine.context().suggestions.remove();
  }
}

export function removeAutoCompleteMenu() {
  machine.send("reset");
}

const styles = document.createElement("link");
styles.rel = "stylesheet";
styles.href = "/static/autocomplete.css";
document.head.appendChild(styles);
