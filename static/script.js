(function () {
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

  function removeLogoutIfNotLoggedIn() {
    if (!document.cookie.login) {
      const footer = document.querySelector(".footer");
      const logout = footer.querySelector('a[href="/logout"]');
      logout.remove();
    }
  }

  replaceOGMeta();
  populateSearch();
  removeLogoutIfNotLoggedIn();

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

  document.onkeydown = function (e) {
    if (e.target !== document.body) return;
    if (e.ctrlKey) return;
    switch (e.key) {
      case "t":
        jump("tasks");
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
