import { assign, StateMachine } from "./utils.js";
import { Scorer } from "./score.js";
import caretPos from "/static/vendors/caretposition.js";

const stateChart = {
  initial: "idle",
  context: {
    completionContext: "",
    focused: false,
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
          actions: [teardownMenu, assign(() => ({ completionContext: "" }))],
        },
      },
    },
    setup: {
      on: {
        done: "completing",
        reset: {
          target: "idle",
          actions: [teardownMenu],
        },
      },
    },
    completing: {
      on: {
        "]": {
          target: "idle",
          actions: [teardownMenu],
        },
        "[": {
          target: "idle",
          actions: [teardownMenu],
        },
        append: {
          target: ".",
          actions: [
            assign((ctx, payload) => {
              return {
                completionContext: ctx.completionContext.concat(payload),
              };
            }),
            updateSuggestionMenu,
          ],
        },
        focus: {
          target: ".",
          actions: [
            assign((_, payload) => ({
              focused: payload,
            })),
          ],
        },
        truncate: {
          target: ".",
          actions: [
            assign((ctx) => {
              return {
                completionContext: ctx.completionContext.slice(
                  0,
                  ctx.completionContext.length - 1
                ),
              };
            }),
            updateSuggestionMenu,
          ],
        },
        reset: {
          target: "idle",
          actions: [
            teardownMenu,
            assign(() => ({ completionContext: "", focused: false })),
          ],
        },
      },
    },
  },
};

const machine = new StateMachine(stateChart);
export function autocomplete(e) {
  const { completionContext, focused } = machine.context();
  switch (e.key) {
    case "[":
      machine.send("[", e);
      break;
    case "]":
      machine.send("]", e);
      break;
    case "Tab":
    case "ArrowDown": {
      if (machine.state === "completing" && !focused) {
        e.preventDefault();
        e.stopImmediatePropagation();
        const menu = document.getElementById("autocomplete-menu");
        const candidate = menu.querySelector(`button[data-idx="0"]`);
        candidate.focus();
        machine.send("focus", true);
      }
      break;
    }
    case "Backspace": {
      if (completionContext.length > 0) {
        machine.send("truncate");
      } else {
        machine.send("reset");
      }
      break;
    }
    default:
      machine.send("append", e.key);
      break;
  }
}

const matcher = new Scorer(3);

function setupMenu(e) {
  caretPos();
  const caret = getCaretCoordinates(e.target, e.target.selectionEnd);
  const suggestions = document.createElement("div");
  suggestions.classList.add("autocomplete-menu");
  suggestions.style.position = "absolute";
  const rect = e.target.getBoundingClientRect();
  suggestions.style.top = `${rect.y + caret.top + caret.height}px`;
  suggestions.style.left = `${caret.left + rect.x}px`;

  const list = document.createElement("ul");
  list.id = "autocomplete-list";

  suggestions.appendChild(list);
  suggestions.id = "autocomplete-menu";
  e.target.parentElement.appendChild(suggestions);
  machine.send("done");
}

async function updateSuggestionMenu() {
  const context = machine.context().completionContext;
  if (context.length < 2) return;
  const candidates = await matcher.test(context);
  const opts = candidates.sort((a, b) => a.score - b.score).slice(0, 15);
  const container = document.getElementById("autocomplete-list");
  const elements = opts.map(({ value: opt }, idx) => {
    const item = document.createElement("li");
    const innerButton = document.createElement("button");
    innerButton.innerText = opt;
    innerButton.setAttribute("data-idx", idx);
    innerButton.addEventListener("click", () => {
      const textarea = document.querySelector("textarea");
      textarea.value =
        textarea.value
          .slice(0, textarea.value.length - context.length)
          .concat(opt) + "]]";
      machine.send("reset");
      textarea.dispatchEvent(new Event("change"));
    });
    item.appendChild(innerButton);
    return item;
  });
  if (container.childElementCount === 0) {
    container.append(...elements);
  } else {
    // TODO: Perf check. Might get slow?
    container.replaceChildren(...elements);
  }
}

export function autocompleteState() {
  return machine.state;
}

function teardownMenu() {
  document.getElementById("autocomplete-menu").remove();
  const textarea = document.querySelector("textarea");
  textarea.focus();
}

export function removeAutoCompleteMenu() {
  machine.send("reset");
}

const styles = document.createElement("link");
styles.rel = "stylesheet";
styles.href = "/static/autocomplete.css";
document.head.appendChild(styles);
