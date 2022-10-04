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
              const nextCtx = action.exec(this.#chart.context, payload);
              this.#chart.context = nextCtx;
              return;
            }
            action.exec(payload);
          });
        this.state = requestedNextState.target;
      }
      this.#processingState = "READY";
      for (const [message, payload] of this.#messageBuffer) {
        this.send(message, payload);
      }
    }
  };
}
