(function () {
  let currentFocusedElement;
  const content = document.getElementById("content-block");
  content.querySelectorAll(".text-block").forEach(function (el) {
    el.addEventListener("click", setupEditor);
  });

  function setupEditor() {
    const textArea = document.createElement("textarea");
    textArea.textContent = this.textContent;
    textArea.addEventListener("keyup", handleInput);
    textArea.addEventListener("keydown", (e) => updateInputHeight(e.target));
    textArea.addEventListener("blur", setupViewer);
    for (const datapoint in this.dataset) {
      textArea.dataset[datapoint] = this.dataset[datapoint];
    }
    this.replaceWith(textArea);
    setAsFocused(textArea);
  }
  function handleInput(e) {
    switch (e.key) {
      case "Backspace": {
        if (e.target.value === "") {
          deleteBlock(e.target);
          break;
        }
        break;
      }
      case "Enter": {
        if (!e.shiftKey) {
          addBlock(e);
          break;
        }
        break;
      }
      default:
        break;
    }
  }
  function setupViewer(e) {
    const div = document.createElement("div");
    div.addEventListener("click", setupEditor);
    div.textContent = e.target.value;
    div.innerHTML = e.target.value.replaceAll("\n", "<br />");
    div.classList.add("text-block");
    if (this.value !== "" && attributesNotSet(e.target)) {
      saveBlock(div, "text/wikitext");
    }
    e.target.replaceWith(div);
  }
  function addBlock(e) {
    const textblock = document.createElement("textarea");
    textblock.addEventListener("blur", setupViewer);
    textblock.addEventListener("keyup", handleInput);
    content.appendChild(textblock);
    setAsFocused(textblock);
  }
  function setAsFocused(el) {
    if (currentFocusedElement) {
      currentFocusedElement.blur();
    }
    currentFocusedElement = el;
    el.focus();
    el.selectionStart = el.textContent?.length;
    updateInputHeight(el);
  }
  function deleteBlock(el) {
    el.remove();
  }
  function updateInputHeight(el) {
    // Let's handle text overflows
    const scrollHeight = el.scrollHeight;
    const clientHeight = el.clientHeight;
    const scrollTop = el.scrollTop;
    const heightDiff = scrollHeight - scrollTop - clientHeight;
    if (heightDiff > 0 || scrollHeight > clientHeight) {
      el.style.height = `${scrollHeight}px`;
    }
  }
  function getDay(dayNumber) {
    if (dayNumber < 10) {
      return `0${dayNumber}`;
    }
    return String(dayNumber);
  }
  function saveBlock(el, type) {
    const now = new Date();
    el.dataset.id = now.valueOf();
    el.dataset.type = type;
    el.dataset.created = `${now.getFullYear()}${now.getMonth() + 1}${getDay(
      now.getDate()
    )}${now.getHours()}${now.getMinutes()}`;
  }
  function attributesNotSet(el) {
    if (
      el.getAttribute("data-id") &&
      el.getAttribute("data-type") &&
      el.getAttribute("data-created")
    ) {
      return false;
    }
    return true;
  }
})();
