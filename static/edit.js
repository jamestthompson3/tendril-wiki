import caretPos from "/static/vendors/caretposition.js";

const states = {
  IDLE: {
    "[": "READY",
  },
  READY: {
    "[": "COMPLETING",
  },
};

let currentState = "IDLE";

let completing = false;
document.querySelector("textarea").addEventListener("keyup", function (e) {
  const derivedState = states[currentState];
  if (e.key === "[") {
    if (derivedState && derivedState["["]) {
      currentState = states[currentState]["["];
    }
    if (currentState === "COMPLETING") {
      caretPos();
      const caret = getCaretCoordinates(this, this.selectionEnd);
      if (completing) {
        return;
      }
      const suggestions = document.createElement("div");
      suggestions.style.position = "absolute";
      const HEIGHT = 100;
      // console.log({scrollHeight: this.scrollHeight, offsetTop: this.offsetTop, caretTop: caret.top});
      console.log({
        scrollWidth: this.scrollWidth,
        caretLeft: caret.left,
        obj: this,
      });
      suggestions.style.top = `${this.scrollHeight}px`;

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

      completing = true;
    }
  }
});
