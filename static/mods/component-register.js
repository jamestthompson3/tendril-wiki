import { htmlToText } from "./parsing.js";
import { StateMachine } from "./utils.js";

const stateChart = {
  initial: "idle",
  states: {
    idle: {
      on: {
        SUBMITTING: "submitting",
      },
    },
    submitting: {
      on: {
        COMPLETE: "idle",
        ERROR: { target: "error", actions: ["showErrors"] },
      },
    },
    error: {
      on: {
        RESET: { target: "idle", actions: ["hideErrors"] },
      },
    },
  },
  actions: {
    showErrors: (msg) => {
      const errorMsg = document.querySelector(".error-msg");
      errorMsg.classList.remove("hidden");
      errorMsg.textContent = `Could not save text: ${msg}`;
    },
    hideErrors: () => {
      const errorMsg = document.querySelector(".error-msg");
      errorMsg.classList.add("hidden");
      errorMsg.textContent = "";
    },
  },
};

export class ComponentRegister {
  #machine;
  constructor() {
    this.components = {};
    try {
      this.bc = new BroadcastChannel(`tendril-wiki${location.pathname}`);
    } catch (e) {
      const failsafe = document.createElement("h1");
      failsafe.innerText = "Your browser doesn't support broadcast channels.";
      failsafe.classList.add("error-msg");
      document.body.firstElementChild.appendChild(failsafe);
      return;
    }
    this.bc.onmessage = this.handleMessage;
    this.#machine = new StateMachine(stateChart);
  }
  handleMessage = (m) => {
    const { data } = m;
    const { data: messageData } = data;
    switch (data.type) {
      case "SAVE":
        if (this.#machine.state === "idle") this.savePage(messageData);
        break;
      case "UNREGISTER":
        this.unregister(messageData);
      case "REGISTER":
        this.register(messageData);
      default:
        break;
    }
  };
  register = (component) => {
    const { id } = component;
    this.components[id] = component;
  };
  unregister = (id) => {
    delete this.components[id];
  };
  getType = (type) =>
    Object.keys(this.components).reduce((arr, key) => {
      if (key.includes(type)) {
        arr.push(this.components[key]);
      }
      return arr;
    }, []);
  dump = () => this.components;
  savePage = (messageData) => {
    this.#machine.send("SUBMITTING");
    if (messageData) {
      this.components[messageData.id].content = messageData.content;
    }
    // TODO: Implement patching API?
    const title = this.components.title.content;
    const body = Array.from(document.querySelectorAll(".text-block"))
      .map((block) => {
        let content = htmlToText(block);
        const indentLevel = parseInt(block.dataset.indent);
        if (indentLevel > 0) {
          content = `${"\t".repeat(indentLevel)}${content}`;
        }
        return content;
      })
      .join("\n");
    const tags = this.components.tag.content;
    const metadata = this.components.metadata.content;
    const constructedBody = {
      body,
      title,
      old_title: CURRENT_TITLE,
      tags,
      metadata,
    };
    fetch("/edit", {
      method: "POST",
      body: JSON.stringify(constructedBody),
      headers: {
        "content-type": "application/json",
      },
    })
      .then((res) => {
        if (res.status < 400) {
          this.#machine.send("COMPLETE");
        }
      })
      .catch((e) => {
        console.error(e);
        this.#machine.send("ERROR", e);
      });
  };
}
