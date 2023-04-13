// Credit to: https://github.com/ai/nanoid
export const nanoid = (size = 21) =>
  crypto.getRandomValues(new Uint8Array(size)).reduce((id, byte) => {
    // It is incorrect to use bytes exceeding the alphabet size.
    // The following mask reduces the random byte in the 0-255 value
    // range to the 0-63 value range. Therefore, adding hacks, such
    // as empty string fallback or magic numbers, is unneccessary because
    // the bitmask trims bytes down to the alphabet size.
    byte &= 63;
    if (byte < 36) {
      // `0-9a-z`
      id += byte.toString(36);
    } else if (byte < 62) {
      // `A-Z`
      id += (byte - 26).toString(36).toUpperCase();
    } else if (byte > 62) {
      id += "-";
    } else {
      id += "_";
    }
    return id;
  }, "");

const ACTION_TYPES = {
  Serialized: "serialized",
  Assign: "assign",
};

export function assign(assign) {
  return { type: ACTION_TYPES.Assign, exec: assign };
}
/**
 *
 * Basic state machine, takes a statechart as input.
 */
export class StateMachine {
  #chart;
  #messageBuffer;
  #processingState;
  constructor(statechart) {
    this.state = statechart.initial;
    this.#chart = statechart;
    this.#messageBuffer = [];
    this.#processingState = "READY";
  }
  transformAction = (action) => {
    if (typeof action === "string") {
      return {
        type: ACTION_TYPES.Serialized,
        exec: this.#chart.actions[action],
      };
    }
    if (typeof action === "function") {
      return { type: action.name, exec: action };
    }
    return action;
  };
  context = () => this.#chart.context;
  send = (message, payload) => {
    if (this.#processingState == "BUSY") {
      this.#messageBuffer.push([message, payload]);
      return;
    }
    const { on } = this.#chart.states[this.state];
    const requestedNextState = on[message];
    if (requestedNextState) {
      this.#processingState = "BUSY";
      if (typeof requestedNextState === "string") {
        this.state = on[message];
      } else {
        if (
          requestedNextState.cond &&
          !requestedNextState.cond(this.#chart.context)
        )
          return;
        requestedNextState.actions
          .map((action) => this.transformAction(action))
          .forEach((action) => {
            if (action.type === ACTION_TYPES.Assign) {
              Object.assign(
                this.#chart.context,
                action.exec(this.#chart.context, payload)
              );
              return;
            }
            action.exec(payload);
          });
        this.state =
          requestedNextState.target === "."
            ? this.state
            : requestedNextState.target;
      }
      this.#processingState = "READY";
      for (const [message, payload] of this.#messageBuffer) {
        this.send(message, payload);
      }
    }
  };
}

const punctMap = {
  // Open-quotes: http://www.fileformat.info/info/unicode/category/Pi/list.htm
  [0x2018]: "'",
  [0x201b]: "'",
  [0x201c]: '"',
  [0x201f]: '"',
  // Close-quotes: http://www.fileformat.info/info/unicode/category/Pf/list.htm
  [0x2019]: "'",
  [0x201d]: '"',
  // Primes: http://www.fileformat.info/info/unicode/category/Po/list.htm
  [0x2032]: "'",
  [0x2033]: '"',
  [0x2035]: "'",
  [0x2036]: '"',
  [0x2014]: "-", // iOS 11 also replaces dashes with em-dash
  [0x2013]: "-", // and "--" with en-dash
};

export function isIOS() {
  return (
    ["iPad", "iPhone", "iPod"].some((p) => navigator.platform.includes(p)) ||
    (navigator.userAgent.includes("Mac") && "ontouchend" in document)
  );
}

/**
 * iOS swaps out perfectly normal, ascii punctuation with the unicode equivalents.
 * We handle unicode just fine, but that doesn't really mean everyone does.
 * If users want to move their notes outside of Tendril, we should be considerate and convert to ascii.
 */
export function normalizePunctuation(node) {
  node.addEventListener("keypress", function (e) {
    if (e.key.length != 1) return;

    const code = e.key.codePointAt(0);
    const replacement = conversionMap[code];
    if (replacement) {
      e.preventDefault();
      document.execCommand("insertText", 0, replacement);
    }
  });
}

/**
 * @typedef {{id: string, [key:string]:any}} NodeData
 */

class ListNode {
  /**
   * @param {NodeData} value
   * @param {ListNode} next
   */
  constructor(value, next) {
    this.value = value;
    this.next = next;
  }
}

export class LinkedList {
  constructor() {
    this.head = null;
    this.tail = null;
  }
  /**
   * @param {NodeData} data
   */
  append(data) {
    const node = new ListNode(data, null);
    // List is empty.
    if (!this.head) {
      this.head = node;
      this.tail = node;
      return this;
    }
    this.tail.next = node;
    this.tail = node;
    return this;
  }
  /**
   * @param {NodeData} value
   * @param {string} nodeId
   */
  insertAfter(data, nodeId) {
    let currentNode = this.head;
    while (currentNode) {
      if (currentNode.value.id === nodeId) {
        const temp = currentNode.next;
        const node = new ListNode(data, temp);
        currentNode.next = node;
        if (this.tail === currentNode) {
          this.tail = node;
        }
        break;
      }
      currentNode = currentNode.next;
    }
    return this;
  }
  delete(nodeId) {
    let currentNode = this.head;
    while (currentNode) {
      if (currentNode.value.id === nodeId) {
        this.head = currentNode.next;
        break;
      }
      if (currentNode.next.value.id === nodeId) {
        const temp = currentNode.next.next;
        if (currentNode.next === this.tail) {
          this.tail = currentNode;
        }
        currentNode.next = temp;
        break;
      }
      currentNode = currentNode.next;
    }
    return this;
  }
  toContentString() {
    let currentNode = this.head;
    let finalString = "";
    while (currentNode) {
      finalString = `${finalString}${currentNode.value.content}`;
      if (currentNode.next) {
        finalString += "\n";
      }
      currentNode = currentNode.next;
    }
    return finalString;
  }
  update(nodeId, content) {
    let currentNode = this.head;
    while (currentNode) {
      if (currentNode.value.id === nodeId) {
        currentNode.value.content = content;
        break;
      }
      currentNode = currentNode.next;
    }
  }
}
