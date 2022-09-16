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

/**
 *
 * Basic state machine, takes a statechart as input.
 */
export class StateMachine {
  #chart;
  constructor(statechart) {
    this.state = statechart.initial;
    this.#chart = statechart;
  }
  send = (message, _payload) => {
    const { on } = this.#chart.states[this.state];
    if (on[message]) {
      this.state = on[message];
    }
  };
}
