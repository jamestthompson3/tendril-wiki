(function () {
  function moveCaretToEnd(el) {
    if (typeof el.selectionStart == "number") {
      el.selectionStart = el.selectionEnd = el.value.length;
    } else if (typeof el.createTextRange != "undefined") {
      el.focus();
      var range = el.createTextRange();
      range.collapse(false);
      range.select();
    }
  }

  function moveCaretToStart(el) {
    if (typeof el.selectionStart == "number") {
      el.selectionStart = el.selectionEnd = 0;
    } else if (typeof el.createTextRange != "undefined") {
      el.focus();
      var range = el.createTextRange();
      range.collapse(false);
      range.select();
    }
  }

  function replaceLinkTo() {
    const linkTo = document.getElementById("linkto");
    if (!linkTo) return;
    // Remove leading '/' of the current note
    const currentWiki = window.location.pathname.slice(1);
    linkTo.href = `/new?linkto=${currentWiki}`;
  }

  replaceLinkTo();

  function replaceOGMeta() {
    const metas = document.querySelectorAll("meta");
    for (const meta of metas) {
      if (meta.attributes.property?.value === "og:url") {
        meta.attributes.content.value = window.location;
      }
    }
  }

  function populateSearch() {
    const params = new URLSearchParams(window.location.search);
    if (params.has("term")) {
      const searchElement = document.getElementById("term");
      if (searchElement) {
        searchElement.value = params.get("term");
      }
    }
  }

  replaceOGMeta();
  populateSearch();

  function edit() {
    const editElement = document.getElementById("edit");
    const editLabel = document.querySelector(
      ".content-container > label:nth-child(1)"
    );
    if (editElement && editLabel) {
      // sometimes the page might not be editable
      editElement.checked = true;
      editLabel.textContent = "Cancel";
    }
  }

  const editCheckBox = document.getElementById("edit");
  if (editCheckBox) {
    editCheckBox.addEventListener("click", clickEdit);
  }

  function clickEdit(e) {
    const editLabel = document.querySelector(
      ".content-container > label:nth-child(1)"
    );
    if (editLabel) {
      if (!e.target.checked) {
        editLabel.textContent = "Edit";
      } else {
        editLabel.textContent = "Cancel";
      }
    }
  }

  function search() {
    const searchElement = document.getElementById("term");
    if (searchElement) {
      searchElement.focus();
      searchElement.scrollIntoView();
    }
  }

  function jump(location) {
    const url = new URL(`/${location}`, window.location.origin);
    window.location.href = url;
  }

  const textarea = document.getElementById("body");
  if (textarea) {
    textarea.onkeydown = function (e) {
      switch (e.key) {
        case "Home":
          moveCaretToStart(textarea);
          break;
        case "End":
          moveCaretToEnd(textarea);
          break;
        default:
          break;
      }
    };

    textarea.addEventListener("paste", detectImagePaste);
  }

  document.onkeydown = function (e) {
    if (e.target !== document.body) return;
    if (e.ctrlKey) return;
    switch (e.key) {
      case "t":
        jump("tasks");
        break;
      case "e":
        edit();
        break;
      case "/":
        search();
        e.preventDefault();
        break;
      case "n":
        jump("new");
        break;
      case "l": {
        // Remove leading '/' of the current note
        const currentWiki = window.location.pathname.slice(1);
        jump(`new?linkto=${currentWiki}`);
        break;
      }
      case "p":
        jump("styles");
        break;
      case "u":
        jump("upload");
        break;
      case "b":
        jump("new_bookmark");
        break;
      default:
        break;
    }
  };

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
            event.target.value += `![${blob.name} *alt text*](/files/${blob.name}) "image title"`;
          })
          .catch((e) => {
            console.error(e);
            event.target.value += "Failed to upload image";
          });
      }
    }
  }
})();
