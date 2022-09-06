import { textToHtml, htmlToText } from "./parsing.js";

let currentFocusedElement;

export function addBlock(indentationLevel) {
  const textblock = document.createElement("textarea");
  if (indentationLevel) {
    textblock.dataset.indent = indentationLevel;
  }
  setupTextblockListeners(textblock, "text-block");
  // insert the new block directly after the current block
  const { parentNode, nextSibling } = this;
  parentNode.insertBefore(textblock, nextSibling);
  setAsFocused(textblock);
}

export function deleteBlock(el) {
  el.remove();
}

export function saveBlock() {
  let fullBody;
  const pageContent = document.getElementById("content-block");
  for (let i = 0; i < pageContent.children.length; i++) {
    const child = pageContent.children[i];
    const text = htmlToText(child);
    if (fullBody) {
      fullBody = `${fullBody}\n${text}`;
    } else {
      fullBody = text;
    }
  }

  const title = document.getElementsByClassName("title")[0].innerText;
  // TODO parse out and save metadata, tags
  fetch("/edit", {
    method: "POST",
    body: `body=${encodeURIComponent(
      fullBody
    )}&title=${title}&old_title=${CURRENT_TITLE}`,
    headers: {
      "content-type": "application/x-www-form-urlencoded",
    },
  })
    .then((res) => {
      if (res.status < 400) {
        if (CURRENT_TITLE !== title) {
          history.pushState({ name: "edit page title" }, "", title);
          const mru = document.getElementById("mru");
          const links = mru.querySelectorAll("a");
          for (const link of links) {
            if (link.href.includes(CURRENT_TITLE)) {
              link.href = link.href.replace(CURRENT_TITLE, title);
              link.innerText = link.innerText.replace(CURRENT_TITLE, title);
            }
          }
          CURRENT_TITLE = title;
        }
      }
    })
    .catch(console.error);
}

export function setAsFocused(el) {
  if (currentFocusedElement) {
    currentFocusedElement.blur();
  }
  currentFocusedElement = el;
  el.focus();
  el.selectionStart = el.textContent?.length;
  updateInputHeight(el);
}

export function updateInputHeight(el) {
  // Let's handle text overflows
  const scrollHeight = el.scrollHeight;
  const clientHeight = el.clientHeight;
  const scrollTop = el.scrollTop;
  const heightDiff = scrollHeight - scrollTop - clientHeight;
  if (heightDiff > 0 || scrollHeight > clientHeight) {
    el.style.height = `${scrollHeight}px`;
  }
}

export function setupViewer(divClass) {
  return function (e) {
    let el;
    const html = textToHtml(e.target.value);
    if (divClass === "title") {
      el = document.createElement("h1");
      el.classList.add("title");
      el.innerHTML = html;
    } else {
      el = document.createElement("div");
      el.innerHTML = html;
      el.classList.add(divClass);
    }
    el.addEventListener("click", setupEditor(divClass));
    for (const datapoint in this.dataset) {
      el.dataset[datapoint] = this.dataset[datapoint];
    }
    e.target.replaceWith(el);
    if (this.value !== "" && this.value !== "\n") {
      saveBlock();
    }
  };
}

export function setupEditor(divClass) {
  return function (e) {
    // don't try to edit the block when we're clicking a link
    if (e.target.nodeName === "A") return;
    let textblock;
    if (divClass === "title") {
      textblock = document.createElement("input");
      textblock.type = "text";
      textblock.value = htmlToText(this);
    } else {
      textblock = document.createElement("textarea");
      textblock.textContent = htmlToText(this);
    }

    for (const datapoint in this.dataset) {
      textblock.dataset[datapoint] = this.dataset[datapoint];
    }
    textblock.classList.add(divClass);
    setupTextblockListeners(textblock, divClass);
    this.replaceWith(textblock);
    setAsFocused(textblock);
  };
}

export function handleInput(e) {
  switch (e.key) {
    case "Backspace": {
      if (e.target.value === "" && e.target.parentNode.children.length > 1) {
        deleteBlock(e.target);
        saveBlock();
        break;
      }
      break;
    }
    case "Enter": {
      if (!e.shiftKey) {
        this.value = this.value.slice(0, this.value.length - 1);
        const indentation = this.dataset.indent;
        addBlock.bind(this)(indentation && Number(indentation));
        break;
      }
      break;
    }
    default:
      break;
  }
}

export function handleKeydown(e) {
  if (e.key === "Tab") {
    // TODO: Figure out if I want to deal with indentation like an outliner.
    // e.preventDefault();
    // const indentation = this.dataset.indent;
    // if (indentation) {
    //   // Max indent is 3 levels, min is 0
    //   const indentationLevel = e.shiftKey
    //     ? Math.max(Number(indentation) - 1, 0)
    //     : Math.min(Number(indentation) + 1, 3);
    //   this.dataset.indent = indentationLevel;
    // } else if (!e.shiftKey) {
    //   this.dataset.indent = 1;
    // }
  } else {
    updateInputHeight(this);
  }
}

function setupTextblockListeners(textblock, divClass) {
  textblock.addEventListener("blur", setupViewer(divClass));
  if (divClass !== "title") {
    textblock.addEventListener("keyup", handleInput);
    textblock.addEventListener("keydown", handleKeydown);
    textblock.addEventListener("paste", detectImagePaste);
  }
}

function detectImagePaste(event) {
  const items = (event.clipboardData || event.originalEvent.clipboardData)
    .items;
  for (let index in items) {
    const item = items[index];
    if (item.kind === "file") {
      // we need to get the filename, and `getAsFile` clobbers this with a generic name
      // so we can just use FormData here.
      const formData = new FormData();
      const file = item.getAsFile();
      const extension = file.type.split("image/").find(Boolean);
      formData.append(
        "file",
        item.getAsFile(),
        `image-${new Date().valueOf()}.${extension}`
      );
      const blob = formData.get("file");
      fetch("/files", {
        method: "POST",
        headers: {
          "Content-Type": "application/octet-stream",
          Filename: `${blob.name}`,
        },
        body: blob,
      })
        .then(() => {
          // TODO figure out alt text...
          event.target.value += `${window.location.origin}/files/${blob.name}`;
        })
        .catch((e) => {
          console.error(e);
          event.target.value += "Failed to upload image";
        });
    }
  }
}
