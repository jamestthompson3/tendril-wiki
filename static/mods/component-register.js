import { htmlToText } from "./parsing.js";
import { StateMachine, LinkedList } from "./utils.js";

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
    this.components = {
      blocks: new LinkedList(),
    };
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
        break;
      case "REGISTER":
        this.register(messageData);
        break;
      default:
        break;
    }
  };
  register = (component) => {
    const { id, content, parent } = component;
    if (this.isBlock(id)) {
      parent
        ? this.components.blocks.insertAfter({ id, content }, parent)
        : this.components.blocks.append({ id, content });
    }
    this.components[id] = component;
  };
  unregister = (id) => {
    if (this.isBlock(id)) {
      this.components.blocks.delete(id);
      return;
    }
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
      if (!this.isBlock(messageData.id)) {
        this.components[messageData.id].content = messageData.content;
      } else {
        this.components.blocks.update(messageData.id, messageData.content);
      }
    }
    // TODO: Implement patching API?
    const title = this.components.title.content;
    const body = this.components.blocks.toContentString();
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
          if (document.title === "New Entry") {
            history.pushState(
              {},
              "",
              encodeURIComponent(constructedBody.title),
            );
          }
          this.#machine.send("COMPLETE");
        }
      })
      .catch((e) => {
        console.error(e);
        this.#machine.send("ERROR", e);
      });
  };
  isBlock = (id) => id.includes("block");
}
