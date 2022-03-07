(function () {
  const SORT_DIR = {
    ASC: "ascending",
    DESC: "descending",
  };
  const statusHeader = document.querySelector("thead > tr > th:first-child");
  statusHeader.onclick = (e) => sortBy(e, status);
  const prioHeader = document.querySelector("thead > tr > th:nth-child(2)");
  prioHeader.onclick = (e) => sortBy(e, priority);

  function sortBy(e, sortFn) {
    // clear aria sort roles on other sortable headers;
    for (const el of document.querySelectorAll("[aria-sort*='ending']")) {
      el !== e.currentTarget && el.setAttribute("aria-sort", undefined);
    }
    let dir;
    const sortDir = e.currentTarget.getAttribute("aria-sort");
    switch (sortDir) {
      case SORT_DIR.ASC:
        dir = SORT_DIR.DESC;
        e.currentTarget.setAttribute("aria-sort", SORT_DIR.DESC);
        break;
      case SORT_DIR.DESC:
        dir = SORT_DIR.ASC;
        e.currentTarget.setAttribute("aria-sort", SORT_DIR.ASC);
        break;
      default:
        dir = SORT_DIR.DESC;
        e.currentTarget.setAttribute("aria-sort", SORT_DIR.DESC);
        break;
    }
    const rowWrapper = document.querySelector("tbody");
    const taskList = Array.from(
      document.querySelectorAll("[role='row']").values()
    ).sort(sortFn(dir));
    for (const task of taskList) {
      rowWrapper.appendChild(task);
    }
  }

  // Sort functions
  // All take a <tr> element as the sort element
  function status(direction) {
    return function (a, b) {
      const statusA = a.querySelector("td:first-child").innerText;
      const statusB = b.querySelector("td:first-child").innerText;
      const dateComp = sortDate(a, b);
      if (direction === SORT_DIR.ASC) {
        if (statusA > statusB) return 1;
        if (statusA < statusB) return -1;
        return 0 + dateComp;
      }
      if (statusB > statusA) return 1;
      if (statusB < statusA) return -1;
      return 0 + dateComp;
    };
  }

  function priority(direction) {
    return function (a, b) {
      const prioA = a.querySelector("td:nth-child(2)").innerText;
      const prioB = b.querySelector("td:nth-child(2)").innerText;
      const dateComp = sortDate(a, b);
      if (direction === SORT_DIR.ASC) {
        if (prioA > prioB) return 1;
        if (prioA < prioB) return -1;
        return 0 + dateComp;
      }
      if (prioA > prioB) return -1;
      if (prioA < prioB) return 1;
      return 0 + dateComp;
    };
  }

  function sortDate(a, b) {
    const dateA = a.querySelector("td:nth-child(3)").innerText;
    const dateB = b.querySelector("td:nth-child(3)").innerText;
    switch (true) {
      case !dateA && !dateB:
        return 0;
      case dateA && !dateB:
        return -1;
      case !dateA && dateB:
        return 1;
      case Boolean(dateA) && Boolean(dateB): {
        const toDateA = new Date(dateA);
        const toDateB = new Date(dateB);
        if (toDateA > toDateB) return -1;
        if (toDateA < toDateB) return 1;
        return 0;
      }
      default:
        return 0;
    }
  }
})();
