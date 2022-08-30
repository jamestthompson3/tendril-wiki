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
})();
