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

  function edit() {
    const editElement = document.getElementById("edit");
    if (editElement) {
      // sometimes the page might not be editable
      editElement.checked = true;
    }
  }

  function search() {
    window.location.pathname = "/search";
  }

  function jumpNew() {
    window.location.pathname = "/new";
  }

  function jumpLink() {
    const url = new URL(`/new?linkto=${buildLinkTo()}`, window.location.origin);
    window.location.href = url;
  }

  function jumpPaint() {
    window.location.pathname = "/styles";
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
  }
  document.onkeydown = function (e) {
    if (e.target !== document.body) return;
    switch (e.key) {
      case "e":
        edit();
        break;
      case "/":
        search();
        break;
      case "n":
        jumpNew();
        break;
      case "l":
        jumpLink();
      case "p":
        jumpPaint();
      default:
        break;
    }
  };

  function buildLinkTo() {
    // Remove leading '/'
    return window.location.pathname.slice(1);
  }

  function replaceLinkTo() {
    const linkTo = document.getElementById("linkto");
    if (!linkTo) return;
    linkTo.href = `/new?linkto=${buildLinkTo()}`;
  }

  replaceLinkTo();
})();

function paster(event) {
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
      console.log(blob);
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

let textarea = document.getElementById("body");

textarea.addEventListener("paste", paster);
